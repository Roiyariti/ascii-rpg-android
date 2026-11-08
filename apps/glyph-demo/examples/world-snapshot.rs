//! Captura reproducible de la escena real mediante GPU, sin servidor de ventanas.
use ascii_renderer::{GlyphRenderer, MAX_GLYPHS, depth_texture};
use ascii_rpg::scene::{Scene, viewport};
use game_core::GameState;
use glam::IVec2;
use std::{io::Write, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or(".artifacts/world-a.ppm");
    let seed = args.get(2).map(|v| v.parse()).transpose()?.unwrap_or(42);
    let x = args.get(3).map(|v| v.parse()).transpose()?.unwrap_or(0);
    let y = args.get(4).map(|v| v.parse()).transpose()?.unwrap_or(0);
    let zoom = args
        .get(5)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(1.25_f32)
        .clamp(0.85, 2.0);
    let state = GameState {
        seed,
        player: IVec2::new(x, y),
        ..Default::default()
    };
    let mut scene = Scene::new(&state);
    scene.camera.zoom = zoom;
    let mut glyphs = Vec::with_capacity(MAX_GLYPHS);
    let (width, height) = (1280, 720);
    scene.build(&mut glyphs, &state, 0.0, 0.02, viewport(width, height));
    assert!(glyphs.len() <= MAX_GLYPHS);

    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = instance.request_adapter(&Default::default()).await?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        })
        .await?;
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Captura terreno"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = depth_texture(&device, width, height);
    GlyphRenderer::new(&device, &queue, format).render(
        &device,
        &queue,
        &target.create_view(&Default::default()),
        &depth.create_view(&Default::default()),
        &glyphs,
    )?;
    let stride = (width * 4).div_ceil(256) * 256;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Captura lectura"),
        size: u64::from(stride) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(height),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(30)),
    })?;
    rx.recv_timeout(Duration::from_secs(30))??;
    let data = readback.slice(..).get_mapped_range()?;
    // Las caras deben cubrir una parte importante de la vista, no solo el HUD.
    let covered = data
        .chunks_exact(stride as usize)
        .flat_map(|row| row[..width as usize * 4].chunks_exact(4))
        .filter(|pixel| {
            pixel[..3]
                .iter()
                .zip([22_u8, 29, 36])
                .any(|(&actual, clear)| actual.abs_diff(clear) > 12)
        })
        .count();
    assert!(
        covered > (width * height / 2) as usize,
        "La captura debe contener terreno visible, no una pantalla vacía"
    );
    if let Some(parent) = std::path::Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(output, "P6\n{width} {height}\n255\n")?;
    for row in data.chunks_exact(stride as usize) {
        for pixel in row[..width as usize * 4].chunks_exact(4) {
            output.write_all(&pixel[..3])?;
        }
    }
    output.flush()?;
    eprintln!(
        "Captura {path}: semilla {seed}, ({x},{y}), zoom {zoom}, {} glifos, {} celdas, {} chunks, {} KiB cache, CPU construcción {:.2}ms; {:?}",
        glyphs.len(),
        scene.stats.tiles,
        scene.stats.chunks,
        scene.stats.cache_bytes / 1024,
        scene.stats.cpu_ms,
        adapter.get_info().backend
    );
    Ok(())
}
