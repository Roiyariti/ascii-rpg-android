//! Prueba real del shader, atlas y lectura GPU; falla si no hay adaptador.
use ascii_renderer::{Glyph, GlyphRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;
    eprintln!("Prueba GPU: {:?}", adapter.get_info());
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_defaults(),
            ..Default::default()
        })
        .await?;
    let extent = wgpu::Extent3d {
        width: 64,
        height: 64,
        depth_or_array_layers: 1,
    };
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Prueba de glifo"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let renderer = GlyphRenderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);
    let glyphs = [Glyph::new(
        'A',
        [16.0, 16.0],
        [32.0; 2],
        [64.0; 2],
        [1.0, 0.0, 0.0, 1.0],
    )];
    renderer.render(
        &device,
        &queue,
        &target.create_view(&Default::default()),
        &glyphs,
    )?;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Lectura GPU"),
        size: 64 * 256,
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
                bytes_per_row: Some(256),
                rows_per_image: Some(64),
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
        timeout: Some(std::time::Duration::from_secs(30)),
    })?;
    rx.recv_timeout(std::time::Duration::from_secs(30))??;
    let data = readback.slice(..).get_mapped_range()?;
    let mut red = 0;
    for (index, pixel) in data.chunks_exact(4).enumerate() {
        if pixel[0] > 240 && pixel[1] < 5 && pixel[2] < 5 {
            let (x, y) = (index % 64, index / 64);
            assert!(
                (16..48).contains(&x) && (16..48).contains(&y),
                "Glifo fuera de su rectángulo"
            );
            red += 1;
        }
    }
    assert_eq!(
        red, 256,
        "La A original tiene 16 puntos, ampliados a 4x4 píxeles"
    );
    assert!(
        data[0] < 10 && data[1] < 10 && data[2] < 10,
        "El fondo debe limpiarse"
    );
    eprintln!("GPU correcta: {red} píxeles de cobertura y límites del glifo verificados");
    Ok(())
}
