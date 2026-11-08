//! Un atlas, un buffer reutilizable y una llamada de dibujo para un lote de glifos.
use bytemuck::{Pod, Zeroable};
use game_assets::{ATLAS_HEIGHT, ATLAS_WIDTH, CELL, atlas, glyph_index};

pub const MAX_GLYPHS: usize = 16_384;
pub const CLEAR: wgpu::Color = wgpu::Color {
    r: 0.008,
    g: 0.012,
    b: 0.018,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Glyph {
    rect: [f32; 4],
    uv: [f32; 4],
    color: [f32; 4],
}

impl Glyph {
    pub fn new(
        character: char,
        position: [f32; 2],
        size: [f32; 2],
        viewport: [f32; 2],
        color: [f32; 4],
    ) -> Self {
        let index = glyph_index(character);
        Self {
            rect: [
                position[0] / viewport[0] * 2.0 - 1.0,
                1.0 - position[1] / viewport[1] * 2.0,
                size[0] / viewport[0] * 2.0,
                -size[1] / viewport[1] * 2.0,
            ],
            uv: [
                (index % 16 * CELL) as f32 / ATLAS_WIDTH as f32,
                (index / 16 * CELL) as f32 / ATLAS_HEIGHT as f32,
                CELL as f32 / ATLAS_WIDTH as f32,
                CELL as f32 / ATLAS_HEIGHT as f32,
            ],
            color,
        }
    }
}

pub struct GlyphRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    instances: wgpu::Buffer,
}

impl GlyphRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let extent = wgpu::Extent3d {
            width: ATLAS_WIDTH,
            height: ATLAS_HEIGHT,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atlas ASCII R8"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &atlas(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ATLAS_WIDTH),
                rows_per_image: Some(ATLAS_HEIGHT),
            },
            extent,
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader ASCII"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/glyph.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Glifos instanciados"), layout: None,
            vertex: wgpu::VertexState {
                module: &shader, entry_point: Some("vs_main"), compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<Glyph>() as u64, step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
                })],
            },
            primitive: wgpu::PrimitiveState::default(), depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader, entry_point: Some("fs_main"), compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None, cache: None,
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ASCII nearest"),
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas compartido"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instancias reutilizables"),
            size: (MAX_GLYPHS * size_of::<Glyph>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            bind_group,
            instances,
        }
    }

    /// Devuelve error antes de escribir si el lote excede el presupuesto del prototipo.
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        glyphs: &[Glyph],
    ) -> Result<(), &'static str> {
        if glyphs.len() > MAX_GLYPHS {
            return Err("Se excedió el presupuesto de glifos");
        }
        queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(glyphs));
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Pase ASCII"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.draw(0..6, 0..glyphs.len() as u32);
        }
        queue.submit([encoder.finish()]);
        Ok(())
    }
}
