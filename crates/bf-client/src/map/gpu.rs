//! GPU side of the map: textures, pipeline and the egui paint callback.

use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};

use super::MapData;

/// Province ids per row in the palette and owner tables.
pub const TABLE_WIDTH: u32 = 256;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Uniforms {
    pub view_origin: [f32; 2],
    pub view_size: [f32; 2],
    pub map_size: [f32; 2],
    pub texel_per_px: f32,
    pub time: f32,
    pub hovered: u32,
    pub selected: u32,
    pub dim: f32,
    /// Terrain shading strength, 0 to 1.
    pub relief: f32,
    /// How far country colours are softened toward paper (political map) or not (map modes).
    pub pastel: f32,
    pub _pad: [f32; 3],
}

/// Lives in egui-wgpu's callback resources for the whole session.
pub struct MapGpu {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    palette: wgpu::Texture,
    /// Version of the palette last written to `palette`.
    palette_version: u64,
    table_rows: u32,
}

impl MapGpu {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: wgpu::TextureFormat,
        data: &MapData,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("map"),
            source: wgpu::ShaderSource::Wgsl(include_str!("map.wgsl").into()),
        });

        let ids = upload(
            device,
            queue,
            "map ids",
            data.width,
            data.height,
            wgpu::TextureFormat::R16Uint,
            bytemuck::cast_slice(&data.ids),
            2,
        );
        let table_rows = (data.province_count + 1).div_ceil(TABLE_WIDTH);
        let palette = upload(
            device,
            queue,
            "map palette",
            TABLE_WIDTH,
            table_rows,
            wgpu::TextureFormat::Rgba8Unorm,
            &vec![0u8; (TABLE_WIDTH * table_rows * 4) as usize],
            4,
        );
        // Per province: owner index in bits 0-14, colony flag in bit 15, state id in 16-31.
        let mut owner_table = vec![0u32; (TABLE_WIDTH * table_rows) as usize];
        for (i, owner) in data.owner_of.iter().enumerate() {
            let colony = if data.colony_of[i] { 0x8000 } else { 0 };
            let state = u32::from(data.state_of[i]) << 16;
            owner_table[i] = u32::from(*owner & 0x7FFF) | colony | state;
        }
        let owners = upload(
            device,
            queue,
            "map owners",
            TABLE_WIDTH,
            table_rows,
            wgpu::TextureFormat::R32Uint,
            bytemuck::cast_slice(&owner_table),
            4,
        );
        let fields = upload(
            device,
            queue,
            "map fields",
            data.field_width,
            data.field_height,
            wgpu::TextureFormat::Rg8Unorm,
            &data.fields,
            2,
        );
        let relief = upload(
            device,
            queue,
            "map relief",
            data.relief.width,
            data.relief.height,
            wgpu::TextureFormat::R8Unorm,
            &data.relief.pixels,
            1,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("map fields"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("map uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tex = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("map"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                tex(1, wgpu::TextureSampleType::Uint),
                tex(2, wgpu::TextureSampleType::Float { filterable: false }),
                tex(3, wgpu::TextureSampleType::Uint),
                tex(4, wgpu::TextureSampleType::Float { filterable: true }),
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex(6, wgpu::TextureSampleType::Float { filterable: true }),
            ],
        });
        let view = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("map"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view(&ids)),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&view(&palette)),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&view(&owners)),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&view(&fields)),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&view(&relief)),
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("map"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("map"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let gpu = Self {
            pipeline,
            bind_group,
            uniforms,
            palette,
            palette_version: 0,
            table_rows,
        };
        gpu.set_palette(queue, &data.palette);
        gpu
    }

    /// Replaces the colour of every province (map modes change only this table).
    pub fn set_palette(&self, queue: &wgpu::Queue, colors: &[[u8; 4]]) {
        let mut table = vec![0u8; (TABLE_WIDTH * self.table_rows * 4) as usize];
        for (i, c) in colors.iter().enumerate() {
            table[i * 4..i * 4 + 4].copy_from_slice(c);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.palette,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &table,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(TABLE_WIDTH * 4),
                rows_per_image: Some(self.table_rows),
            },
            wgpu::Extent3d {
                width: TABLE_WIDTH,
                height: self.table_rows,
                depth_or_array_layers: 1,
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    bytes: &[u8],
    bytes_per_texel: u32,
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * bytes_per_texel),
            rows_per_image: Some(height),
        },
        size,
    );
    texture
}

/// One frame's draw of the map.
pub struct MapCallback {
    pub uniforms: Uniforms,
    /// Province colours and their version; uploaded when the version is new.
    pub palette: std::sync::Arc<Vec<[u8; 4]>>,
    pub palette_version: u64,
}

impl CallbackTrait for MapCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(gpu) = resources.get_mut::<MapGpu>() {
            queue.write_buffer(&gpu.uniforms, 0, bytemuck::bytes_of(&self.uniforms));
            if gpu.palette_version != self.palette_version {
                gpu.set_palette(queue, &self.palette);
                gpu.palette_version = self.palette_version;
            }
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(gpu) = resources.get::<MapGpu>() {
            pass.set_pipeline(&gpu.pipeline);
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
