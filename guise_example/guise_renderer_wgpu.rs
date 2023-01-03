use std::borrow::Cow;
use std::collections::HashMap;
use std::mem;
use std::num::{NonZeroU32, NonZeroU64};

use wgpu::util::DeviceExt as _;

struct TextureResource {
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Zeroable, bytemuck::Pod)]
struct TransformUniforms {
    matrix: [[f32; 4]; 4],
}

pub struct Renderer {
    sampler: wgpu::Sampler,
    transform_uniform_buffer: wgpu::Buffer,
    transform_uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,

    texture_resources: HashMap<u64, TextureResource>,
    texture_resources_next_id: u64,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, render_attachment_format: wgpu::TextureFormat) -> Self {
        // static SHADER_SOURCE: &str = include_str!("../guise_example/guise_renderer_wgpu.wgsl");
        static VS_SOURCE: &[u32] =
            vk_shader_macros::include_glsl!("../guise_example/guise_renderer_wgpu.vert");
        static FS_SOURCE: &[u32] =
            vk_shader_macros::include_glsl!("../guise_example/guise_renderer_wgpu.frag");

        let vs_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(VS_SOURCE)),
        });
        let fs_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::from(FS_SOURCE)),
        });

        // Create transform uniform buffer bind group
        let transform_uniform_size = size_of::<TransformUniforms>();
        let transform_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: transform_uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(transform_uniform_size).unwrap()),
                    },
                    count: None,
                }],
            });

        let transform_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &transform_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &transform_uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // Create texture uniform bind group
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create render pipeline

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &transform_uniform_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Setup render state: alpha-blending enabled, no face
        // culling, no depth testing

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_shader_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<guise::Vertex>(),
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // a_position
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        // a_tex_coord
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        // a_color
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_shader_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_attachment_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        Self {
            sampler,
            render_pipeline,
            transform_uniform_buffer,
            transform_uniform_bind_group,
            texture_bind_group_layout,

            texture_resources: HashMap::new(),
            texture_resources_next_id: 0,
        }
    }

    pub fn add_texture_rgba8_unorm(
        &mut self,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> u64 {
        assert_eq!(data.len() % 4, 0);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(4 * width).unwrap()),
                rows_per_image: Some(NonZeroU32::new(height).unwrap()),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let texture_id = self.texture_resources_next_id;
        self.texture_resources_next_id += 1;

        self.texture_resources.insert(texture_id, TextureResource {
            bind_group: texture_bind_group,
        });

        texture_id
    }

    pub fn remove_texture(&mut self, id: u64) {
        self.texture_resources.remove(&id);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_attachment: &wgpu::TextureView,
        clear_color: wgpu::Color,
        viewport_physical_width: u32,
        viewport_physical_height: u32,
        viewport_scale: f32,
        commands: &[guise::Command],
        vertices: &[guise::Vertex],
        indices: &[u32],
    ) {
        if commands.is_empty() || vertices.is_empty() || indices.is_empty() {
            return;
        }

        if viewport_physical_width == 0 || viewport_physical_height == 0 {
            return;
        }

        // TODO(yan): @Speed Staging Belt, or re-use buffer.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let transform = {
            // Setup orthographic projection matrix.
            let l = 0.0;
            let r = viewport_physical_width as f32 / viewport_scale;
            let t = 0.0;
            let b = viewport_physical_height as f32 / viewport_scale;

            #[rustfmt::skip]
            let matrix = [
                [2.0 / (r - l)    , 0.0              , 0.0, 0.0],
                [0.0              , 2.0 / (t - b)    , 0.0, 0.0],
                [0.0              , 0.0              , 0.5, 0.0],
                [(r + l) / (l - r), (t + b) / (b - t), 0.5, 1.0],
            ];

            TransformUniforms { matrix }
        };

        queue.write_buffer(
            &self.transform_uniform_buffer,
            0,
            bytemuck::bytes_of(&transform),
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_attachment,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);

        let vw = viewport_physical_width;
        let vh = viewport_physical_height;

        let mut consumed_index_count: u32 = 0;
        for command in commands {
            let x = f32::floor(viewport_scale * command.scissor_rect.x) as u32;
            let y = f32::floor(viewport_scale * command.scissor_rect.y) as u32;
            let w = f32::round(viewport_scale * command.scissor_rect.width) as u32;
            let h = f32::round(viewport_scale * command.scissor_rect.height) as u32;

            if w == 0 || h == 0 || x + w > vw || y + h > vh {
                log::error!("Scissor rect ({x} {y} {w} {h}) invalid");
                continue;
            }

            let texture_resource = match self.texture_resources.get(&command.texture_id) {
                Some(texture_resource) => texture_resource,
                None => {
                    log::error!("Missing texture {}", command.texture_id);
                    continue;
                }
            };

            render_pass.set_scissor_rect(x, y, w, h);
            render_pass.set_bind_group(1, &texture_resource.bind_group, &[]);
            render_pass.draw_indexed(
                consumed_index_count..(consumed_index_count + command.index_count),
                0,
                0..1,
            );

            consumed_index_count += command.index_count;
        }
    }
}

fn size_of<T>() -> wgpu::BufferAddress {
    let size = mem::size_of::<T>();
    wgpu::BufferAddress::try_from(size)
        .unwrap_or_else(|_| panic!("Size {size} does not fit into wgpu BufferAddress"))
}
