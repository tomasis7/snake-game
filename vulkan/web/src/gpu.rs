//! wgpu renderer: WebGPU first, WebGL2 as the fallback. One pipeline, one instanced draw.

use furious_core::atlas::ATLAS_SIZE;
use furious_core::instance::Instance;
use furious_core::viewport::{CANVAS_H, CANVAS_W};
use wgpu::util::DeviceExt;

const SHADER: &str = include_str!("../../shaders/quad.wgsl");
const CANVAS_PX: (u32, u32) = (CANVAS_W as u32, CANVAS_H as u32);

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    instance_buf: wgpu::Buffer,
    offscreen_view: wgpu::TextureView,
    /// "webgpu" or "webgl2".
    pub api: &'static str,
    pub gpu_name: String,
}

pub fn as_bytes(v: &[Instance]) -> &[u8] {
    // Instance is #[repr(C)], 64 bytes of plain f32s (asserted in core).
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

fn has_navigator_gpu() -> bool {
    web_sys::window().is_some_and(|w| js_sys::Reflect::get(&w.navigator(), &"gpu".into()).is_ok_and(|g| !g.is_undefined() && !g.is_null()))
}

fn instance_for(backends: wgpu::Backends) -> wgpu::Instance {
    wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..Default::default() })
}

impl Gpu {
    /// Tries WebGPU (needs `navigator.gpu` and an adapter), then WebGL2.
    pub async fn new(canvas: web_sys::HtmlCanvasElement, capacity: usize, atlas_rgba: &[u8]) -> Result<Gpu, String> {
        if has_navigator_gpu() {
            let instance = instance_for(wgpu::Backends::BROWSER_WEBGPU);
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await;
            if let Ok(adapter) = adapter {
                let surface = instance
                    .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
                    .map_err(|e| format!("WebGPU surface: {e}"))?;
                return Self::finish(adapter, surface, "webgpu", wgpu::Limits::default(), capacity, atlas_rgba).await;
            }
        }
        // WebGL2: the context is created together with the surface, so the surface comes first.
        let instance = instance_for(wgpu::Backends::GL);
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| format!("WebGL2 surface: {e}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| format!("no WebGPU or WebGL2 adapter available: {e}"))?;
        let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        Self::finish(adapter, surface, "webgl2", limits, capacity, atlas_rgba).await
    }

    async fn finish(
        adapter: wgpu::Adapter,
        surface: wgpu::Surface<'static>,
        api: &'static str,
        limits: wgpu::Limits,
        capacity: usize,
        atlas_rgba: &[u8],
    ) -> Result<Gpu, String> {
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(|e| format!("device request failed: {e}"))?;
        let gpu_name = adapter.get_info().name;

        // Non-sRGB (Unorm) surface so colours are plain sRGB values, like the Vulkan build.
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| matches!(f, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm))
            .or_else(|| caps.formats.iter().copied().find(|f| !f.is_srgb()))
            .or_else(|| caps.formats.first().copied())
            .ok_or("surface reports no formats")?;
        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: CANVAS_PX.0,
                height: CANVAS_PX.1,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            },
        );

        let size = wgpu::Extent3d { width: ATLAS_SIZE as u32, height: ATLAS_SIZE as u32, depth_or_array_layers: 1 };
        let atlas = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("atlas"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            atlas_rgba,
        );
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // canvasSize (16 bytes: vec2 + padding).
        let mut ub = [0u8; 16];
        ub[0..4].copy_from_slice(&CANVAS_W.to_le_bytes());
        ub[4..8].copy_from_slice(&CANVAS_H.to_le_bytes());
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("canvas"),
            contents: &ub,
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&atlas_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let attrs = wgpu::vertex_attr_array![
            0 => Float32x2, 1 => Float32x2, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let offscreen = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d { width: CANVAS_PX.0, height: CANVAS_PX.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let offscreen_view = offscreen.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Gpu { device, queue, surface, pipeline, bind_group, instance_buf, offscreen_view, api, gpu_name })
    }

    /// Uploads the first `n` instances.
    pub fn upload(&self, instances: &[Instance], n: usize) {
        self.queue.write_buffer(&self.instance_buf, 0, as_bytes(&instances[..n]));
    }

    fn draw_into(&self, view: &wgpu::TextureView, n: usize) -> wgpu::CommandBuffer {
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.instance_buf.slice(..));
            pass.draw(0..6, 0..n as u32);
        }
        enc.finish()
    }

    /// Draws the uploaded instances to the canvas.
    pub fn render_surface(&self, n: usize) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.queue.submit([self.draw_into(&view, n)]);
        frame.present();
    }

    /// Draws the uploaded instances into the offscreen texture (bench).
    pub fn render_offscreen(&self, n: usize) {
        self.queue.submit([self.draw_into(&self.offscreen_view, n)]);
    }
}
