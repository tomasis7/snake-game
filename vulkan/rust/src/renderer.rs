//! Raw Vulkan renderer (ash). One instanced-quad pipeline, no vertex buffer.

use ash::{vk, Entry};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::{c_char, c_void, CStr};
use std::io::Cursor;

const FRAMES_IN_FLIGHT: usize = 2;
const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// #14141c converted to linear for the sRGB swapchain.
fn clear_color() -> [f32; 4] {
    [srgb_to_linear(0x14 as f32 / 255.0), srgb_to_linear(0x14 as f32 / 255.0), srgb_to_linear(0x1c as f32 / 255.0), 1.0]
}

static VERT_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/quad.vert.spv"));
static FRAG_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/quad.frag.spv"));

/// Per-instance data: 32 bytes, matches the vertex input layout.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Instance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}

struct FrameData {
    fence: vk::Fence,
    image_available: vk::Semaphore,
    cmd: vk::CommandBuffer,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapped: *mut Instance,
}

pub struct Renderer {
    _entry: Entry,
    instance: ash::Instance,
    debug: Option<(ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
    surface_loader: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    phys: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    views: Vec<vk::ImageView>,
    framebuffers: Vec<vk::Framebuffer>,
    render_finished: Vec<vk::Semaphore>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pool: vk::CommandPool,
    frames: Vec<FrameData>,
    capacity: usize,
    frame: usize,
    dirty: bool,
    pub gpu_name: String,
}

unsafe extern "system" fn debug_cb(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _ty: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user: *mut c_void,
) -> vk::Bool32 {
    let msg = CStr::from_ptr((*data).p_message).to_string_lossy();
    eprintln!("[vulkan {severity:?}] {msg}");
    vk::FALSE
}

pub fn present_mode_name(m: vk::PresentModeKHR) -> &'static str {
    match m {
        vk::PresentModeKHR::IMMEDIATE => "IMMEDIATE",
        vk::PresentModeKHR::MAILBOX => "MAILBOX",
        vk::PresentModeKHR::FIFO => "FIFO",
        vk::PresentModeKHR::FIFO_RELAXED => "FIFO_RELAXED",
        _ => "OTHER",
    }
}

impl Renderer {
    /// `max_instances` is the capacity of each per-frame instance buffer.
    /// `bench` selects the present mode preference (IMMEDIATE > MAILBOX > FIFO vs. FIFO).
    pub fn new(
        window: &winit::window::Window,
        max_instances: usize,
        bench: bool,
    ) -> Result<Self, String> {
        unsafe { Self::new_inner(window, max_instances, bench).map_err(|e| e.to_string()) }
    }

    unsafe fn new_inner(
        window: &winit::window::Window,
        max_instances: usize,
        bench: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let entry = Entry::load()?;
        let display = window.display_handle()?.as_raw();
        let wh = window.window_handle()?.as_raw();

        // Instance + optional validation (debug builds only).
        let mut layers: Vec<*const c_char> = Vec::new();
        let mut exts: Vec<*const c_char> = ash_window::enumerate_required_extensions(display)?.to_vec();
        let mut use_validation = false;
        if cfg!(debug_assertions) {
            let avail = entry.enumerate_instance_layer_properties()?;
            use_validation = avail
                .iter()
                .any(|l| l.layer_name_as_c_str().map(|n| n == VALIDATION_LAYER).unwrap_or(false));
            if use_validation {
                layers.push(VALIDATION_LAYER.as_ptr());
                exts.push(ash::ext::debug_utils::NAME.as_ptr());
            } else {
                eprintln!("warning: validation layer not available");
            }
        }
        let app = vk::ApplicationInfo::default()
            .application_name(c"Vulkan Snake (Rust)")
            .api_version(vk::API_VERSION_1_1);
        let ici = vk::InstanceCreateInfo::default()
            .application_info(&app)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&exts);
        let instance = entry.create_instance(&ici, None)?;

        let debug = if use_validation {
            let du = ash::ext::debug_utils::Instance::new(&entry, &instance);
            let info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(debug_cb));
            let m = du.create_debug_utils_messenger(&info, None)?;
            Some((du, m))
        } else {
            None
        };

        let surface = ash_window::create_surface(&entry, &instance, display, wh, None)?;
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        // Physical device: DISCRETE > INTEGRATED > others; never CPU.
        let mut best: Option<(u32, vk::PhysicalDevice, u32)> = None;
        for pd in instance.enumerate_physical_devices()? {
            let props = instance.get_physical_device_properties(pd);
            let rank = match props.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                vk::PhysicalDeviceType::CPU => continue,
                _ => 1,
            };
            let has_swapchain = instance
                .enumerate_device_extension_properties(pd)?
                .iter()
                .any(|e| e.extension_name_as_c_str().map(|n| n == ash::khr::swapchain::NAME).unwrap_or(false));
            if !has_swapchain {
                continue;
            }
            let qfs = instance.get_physical_device_queue_family_properties(pd);
            let mut qf = None;
            for (i, q) in qfs.iter().enumerate() {
                if q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_loader.get_physical_device_surface_support(pd, i as u32, surface)?
                {
                    qf = Some(i as u32);
                    break;
                }
            }
            if let Some(qf) = qf {
                if best.map_or(true, |(r, _, _)| rank > r) {
                    best = Some((rank, pd, qf));
                }
            }
        }
        let (_, phys, qfi) = best.ok_or("no suitable GPU found")?;
        let props = instance.get_physical_device_properties(phys);
        let gpu_name = props.device_name_as_c_str()?.to_string_lossy().into_owned();

        let prio = [1.0f32];
        let qci = [vk::DeviceQueueCreateInfo::default().queue_family_index(qfi).queue_priorities(&prio)];
        let dev_exts = [ash::khr::swapchain::NAME.as_ptr()];
        let dci = vk::DeviceCreateInfo::default()
            .queue_create_infos(&qci)
            .enabled_extension_names(&dev_exts);
        let device = instance.create_device(phys, &dci, None)?;
        let queue = device.get_device_queue(qfi, 0);
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        // Surface format / present mode.
        let formats = surface_loader.get_physical_device_surface_formats(phys, surface)?;
        let format = formats
            .iter()
            .copied()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
            .unwrap_or(formats[0]);
        let modes = surface_loader.get_physical_device_surface_present_modes(phys, surface)?;
        let present_mode = if bench {
            [vk::PresentModeKHR::IMMEDIATE, vk::PresentModeKHR::MAILBOX]
                .into_iter()
                .find(|m| modes.contains(m))
                .unwrap_or(vk::PresentModeKHR::FIFO)
        } else {
            vk::PresentModeKHR::FIFO
        };

        // Render pass.
        let attachments = [vk::AttachmentDescription::default()
            .format(format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];
        let color_ref = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let subpasses = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_ref)];
        let deps = [vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];
        let render_pass = device.create_render_pass(
            &vk::RenderPassCreateInfo::default()
                .attachments(&attachments)
                .subpasses(&subpasses)
                .dependencies(&deps),
            None,
        )?;

        // Pipeline.
        let vert = device.create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&ash::util::read_spv(&mut Cursor::new(VERT_SPV))?),
            None,
        )?;
        let frag = device.create_shader_module(
            &vk::ShaderModuleCreateInfo::default().code(&ash::util::read_spv(&mut Cursor::new(FRAG_SPV))?),
            None,
        )?;
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag)
                .name(c"main"),
        ];
        let bindings = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Instance>() as u32)
            .input_rate(vk::VertexInputRate::INSTANCE)];
        let attrs = [
            vk::VertexInputAttributeDescription::default().location(0).binding(0).format(vk::Format::R32G32_SFLOAT).offset(0),
            vk::VertexInputAttributeDescription::default().location(1).binding(0).format(vk::Format::R32G32_SFLOAT).offset(8),
            vk::VertexInputAttributeDescription::default().location(2).binding(0).format(vk::Format::R32G32B32A32_SFLOAT).offset(16),
        ];
        let vi = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attrs);
        let ia = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let vp = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let rs = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);
        let ms = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_att = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let cb = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_att);
        let dyn_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dy = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dyn_states);
        let pipeline_layout = device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)?;
        let gpi = [vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vi)
            .input_assembly_state(&ia)
            .viewport_state(&vp)
            .rasterization_state(&rs)
            .multisample_state(&ms)
            .color_blend_state(&cb)
            .dynamic_state(&dy)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)];
        let pipeline = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &gpi, None)
            .map_err(|(_, e)| e)?[0];
        device.destroy_shader_module(vert, None);
        device.destroy_shader_module(frag, None);

        // Command pool, per-frame resources.
        let pool = device.create_command_pool(
            &vk::CommandPoolCreateInfo::default()
                .queue_family_index(qfi)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
            None,
        )?;
        let cmds = device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(FRAMES_IN_FLIGHT as u32),
        )?;
        let mem_props = instance.get_physical_device_memory_properties(phys);
        let capacity = max_instances.max(1);
        let buf_size = (capacity * std::mem::size_of::<Instance>()) as u64;
        let mut frames = Vec::new();
        for cmd in cmds {
            let buffer = device.create_buffer(
                &vk::BufferCreateInfo::default()
                    .size(buf_size)
                    .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE),
                None,
            )?;
            let req = device.get_buffer_memory_requirements(buffer);
            let want = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let mt = (0..mem_props.memory_type_count)
                .find(|&i| {
                    req.memory_type_bits & (1 << i) != 0
                        && mem_props.memory_types[i as usize].property_flags.contains(want)
                })
                .ok_or("no host-visible memory type")?;
            let memory = device.allocate_memory(
                &vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt),
                None,
            )?;
            device.bind_buffer_memory(buffer, memory, 0)?;
            let mapped = device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())? as *mut Instance;
            frames.push(FrameData {
                fence: device.create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )?,
                image_available: device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?,
                cmd,
                buffer,
                memory,
                mapped,
            });
        }

        let mut r = Renderer {
            _entry: entry,
            instance,
            debug,
            surface_loader,
            surface,
            phys,
            device,
            queue,
            swapchain_loader,
            swapchain: vk::SwapchainKHR::null(),
            format,
            extent: vk::Extent2D { width: 0, height: 0 },
            present_mode,
            views: Vec::new(),
            framebuffers: Vec::new(),
            render_finished: Vec::new(),
            render_pass,
            pipeline_layout,
            pipeline,
            pool,
            frames,
            capacity,
            frame: 0,
            dirty: false,
            gpu_name,
        };
        r.recreate_swapchain((window.inner_size().width, window.inner_size().height))?;
        Ok(r)
    }

    pub fn present_mode_name(&self) -> &'static str {
        present_mode_name(self.present_mode)
    }

    pub fn mark_resized(&mut self) {
        self.dirty = true;
    }

    unsafe fn destroy_swapchain_resources(&mut self) {
        for &f in &self.framebuffers {
            self.device.destroy_framebuffer(f, None);
        }
        for &v in &self.views {
            self.device.destroy_image_view(v, None);
        }
        for &s in &self.render_finished {
            self.device.destroy_semaphore(s, None);
        }
        self.framebuffers.clear();
        self.views.clear();
        self.render_finished.clear();
    }

    /// Returns Ok(false) when the surface has zero size (minimized) and nothing was created.
    unsafe fn recreate_swapchain(&mut self, win_size: (u32, u32)) -> Result<bool, vk::Result> {
        self.device.device_wait_idle()?;
        let caps = self.surface_loader.get_physical_device_surface_capabilities(self.phys, self.surface)?;
        let extent = if caps.current_extent.width != u32::MAX {
            caps.current_extent
        } else {
            vk::Extent2D {
                width: win_size.0.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                height: win_size.1.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
            }
        };
        if extent.width == 0 || extent.height == 0 {
            return Ok(false);
        }
        let mut count = caps.min_image_count + 1;
        if caps.max_image_count > 0 {
            count = count.min(caps.max_image_count);
        }
        let alpha = [
            vk::CompositeAlphaFlagsKHR::OPAQUE,
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
            vk::CompositeAlphaFlagsKHR::INHERIT,
        ]
        .into_iter()
        .find(|a| caps.supported_composite_alpha.contains(*a))
        .unwrap_or(vk::CompositeAlphaFlagsKHR::OPAQUE);
        let old = self.swapchain;
        let sci = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(count)
            .image_format(self.format.format)
            .image_color_space(self.format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(alpha)
            .present_mode(self.present_mode)
            .clipped(true)
            .old_swapchain(old);
        let new = self.swapchain_loader.create_swapchain(&sci, None)?;
        self.destroy_swapchain_resources();
        if old != vk::SwapchainKHR::null() {
            self.swapchain_loader.destroy_swapchain(old, None);
        }
        self.swapchain = new;
        self.extent = extent;
        let images = self.swapchain_loader.get_swapchain_images(new)?;
        for &img in &images {
            let view = self.device.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(img)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.format.format)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .layer_count(1),
                    ),
                None,
            )?;
            self.views.push(view);
            let atts = [view];
            self.framebuffers.push(self.device.create_framebuffer(
                &vk::FramebufferCreateInfo::default()
                    .render_pass(self.render_pass)
                    .attachments(&atts)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1),
                None,
            )?);
            self.render_finished
                .push(self.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?);
        }
        self.dirty = false;
        Ok(true)
    }

    /// Render one frame. `fill` writes instances into the mapped buffer (capacity given)
    /// and returns how many were written. `win_size` is the current window size in pixels.
    pub fn render<F: FnOnce(&mut [Instance]) -> usize>(&mut self, win_size: (u32, u32), fill: F) -> Result<(), String> {
        unsafe { self.render_inner(win_size, fill).map_err(|e| e.to_string()) }
    }

    unsafe fn render_inner<F: FnOnce(&mut [Instance]) -> usize>(
        &mut self,
        win_size: (u32, u32),
        fill: F,
    ) -> Result<(), vk::Result> {
        if win_size.0 == 0 || win_size.1 == 0 {
            return Ok(()); // minimized
        }
        if self.dirty
            || self.swapchain == vk::SwapchainKHR::null()
            || (win_size.0 != self.extent.width || win_size.1 != self.extent.height)
        {
            if !self.recreate_swapchain(win_size)? {
                return Ok(());
            }
        }
        let fi = self.frame;
        let fence = self.frames[fi].fence;
        self.device.wait_for_fences(&[fence], true, u64::MAX)?;

        let (image_index, suboptimal) = match self.swapchain_loader.acquire_next_image(
            self.swapchain,
            u64::MAX,
            self.frames[fi].image_available,
            vk::Fence::null(),
        ) {
            Ok(r) => r,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.dirty = true;
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        self.device.reset_fences(&[fence])?;

        let f = &self.frames[fi];
        let slice = std::slice::from_raw_parts_mut(f.mapped, self.capacity);
        let n = fill(slice).min(self.capacity);

        let cmd = f.cmd;
        self.device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())?;
        self.device.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))?;
        let clear = [vk::ClearValue { color: vk::ClearColorValue { float32: clear_color() } }];
        self.device.cmd_begin_render_pass(
            cmd,
            &vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D::default(), extent: self.extent })
                .clear_values(&clear),
            vk::SubpassContents::INLINE,
        );
        self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        self.device.cmd_set_viewport(
            cmd,
            0,
            &[vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.extent.width as f32,
                height: self.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
        );
        self.device.cmd_set_scissor(cmd, 0, &[vk::Rect2D { offset: vk::Offset2D::default(), extent: self.extent }]);
        self.device.cmd_bind_vertex_buffers(cmd, 0, &[f.buffer], &[0]);
        self.device.cmd_draw(cmd, 6, n as u32, 0, 0);
        self.device.cmd_end_render_pass(cmd);
        self.device.end_command_buffer(cmd)?;

        let wait = [f.image_available];
        let stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmds = [cmd];
        let signal = [self.render_finished[image_index as usize]];
        let submit = [vk::SubmitInfo::default()
            .wait_semaphores(&wait)
            .wait_dst_stage_mask(&stages)
            .command_buffers(&cmds)
            .signal_semaphores(&signal)];
        self.device.queue_submit(self.queue, &submit, fence)?;

        let swapchains = [self.swapchain];
        let indices = [image_index];
        let present = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal)
            .swapchains(&swapchains)
            .image_indices(&indices);
        match self.swapchain_loader.queue_present(self.queue, &present) {
            Ok(sub) => {
                if sub || suboptimal {
                    self.dirty = true;
                }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => self.dirty = true,
            Err(e) => return Err(e),
        }
        self.frame = (self.frame + 1) % FRAMES_IN_FLIGHT;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.destroy_swapchain_resources();
            if self.swapchain != vk::SwapchainKHR::null() {
                self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            }
            for f in &self.frames {
                self.device.destroy_fence(f.fence, None);
                self.device.destroy_semaphore(f.image_available, None);
                self.device.unmap_memory(f.memory);
                self.device.destroy_buffer(f.buffer, None);
                self.device.free_memory(f.memory, None);
            }
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            if let Some((du, m)) = self.debug.take() {
                du.destroy_debug_utils_messenger(m, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
