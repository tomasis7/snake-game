//! Raw Vulkan renderer (ash). One instanced-quad pipeline, no vertex buffer, one atlas
//! texture, UNORM swapchain, letterboxed 1200 x 800 canvas, optional screenshot capture.

use ash::{vk, Entry};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::{c_char, c_void, CStr};
use std::io::Cursor;

use furious_core::atlas::ATLAS_SIZE;
use furious_core::instance::Instance;
use furious_core::viewport::{Letterbox, CANVAS_H, CANVAS_W};

const FRAMES_IN_FLIGHT: usize = 2;
const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

static VERT_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/quad.vert.spv"));
static FRAG_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/quad.frag.spv"));

/// A captured canvas, RGBA8.
pub struct Captured {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
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
    can_capture: bool,
    views: Vec<vk::ImageView>,
    images: Vec<vk::Image>,
    framebuffers: Vec<vk::Framebuffer>,
    render_finished: Vec<vk::Semaphore>,
    render_pass: vk::RenderPass,
    set_layout: vk::DescriptorSetLayout,
    desc_pool: vk::DescriptorPool,
    desc_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pool: vk::CommandPool,
    frames: Vec<FrameData>,
    capacity: usize,
    frame: usize,
    dirty: bool,
    mem_props: vk::PhysicalDeviceMemoryProperties,
    atlas_image: vk::Image,
    atlas_memory: vk::DeviceMemory,
    atlas_view: vk::ImageView,
    sampler: vk::Sampler,
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

fn find_memory_type(
    props: &vk::PhysicalDeviceMemoryProperties,
    type_bits: u32,
    want: vk::MemoryPropertyFlags,
) -> Result<u32, String> {
    (0..props.memory_type_count)
        .find(|&i| type_bits & (1 << i) != 0 && props.memory_types[i as usize].property_flags.contains(want))
        .ok_or_else(|| format!("no memory type for {want:?}"))
}

impl Renderer {
    /// `max_instances` is the capacity of each per-frame instance buffer. `bench` selects the
    /// present mode preference (IMMEDIATE > MAILBOX > FIFO versus FIFO). `atlas_rgba` is the
    /// ATLAS_SIZE^2 RGBA8 atlas, uploaded once.
    pub fn new(
        window: &winit::window::Window,
        max_instances: usize,
        bench: bool,
        atlas_rgba: &[u8],
    ) -> Result<Self, String> {
        unsafe { Self::new_inner(window, max_instances, bench, atlas_rgba).map_err(|e| e.to_string()) }
    }

    unsafe fn new_inner(
        window: &winit::window::Window,
        max_instances: usize,
        bench: bool,
        atlas_rgba: &[u8],
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
            .application_name(c"Furious Snake (Rust)")
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
                    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
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

        // Physical device: DISCRETE > INTEGRATED > others; never CPU (llvmpipe).
        let mut best: Option<(u32, vk::PhysicalDevice, u32)> = None;
        for pd in instance.enumerate_physical_devices()? {
            let props = instance.get_physical_device_properties(pd);
            let rank = match props.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                vk::PhysicalDeviceType::CPU => continue,
                _ => 1,
            };
            let has_swapchain = instance.enumerate_device_extension_properties(pd)?.iter().any(|e| {
                e.extension_name_as_c_str().map(|n| n == ash::khr::swapchain::NAME).unwrap_or(false)
            });
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
        let mem_props = instance.get_physical_device_memory_properties(phys);

        let prio = [1.0f32];
        let qci = [vk::DeviceQueueCreateInfo::default().queue_family_index(qfi).queue_priorities(&prio)];
        let dev_exts = [ash::khr::swapchain::NAME.as_ptr()];
        let dci = vk::DeviceCreateInfo::default().queue_create_infos(&qci).enabled_extension_names(&dev_exts);
        let device = instance.create_device(phys, &dci, None)?;
        let queue = device.get_device_queue(qfi, 0);
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        // Surface format: prefer B8G8R8A8_UNORM (plain sRGB values, no linearising).
        let formats = surface_loader.get_physical_device_surface_formats(phys, surface)?;
        let format = formats
            .iter()
            .copied()
            .find(|f| f.format == vk::Format::B8G8R8A8_UNORM)
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
        let caps = surface_loader.get_physical_device_surface_capabilities(phys, surface)?;
        let can_capture = caps.supported_usage_flags.contains(vk::ImageUsageFlags::TRANSFER_SRC);

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
        let color_ref =
            [vk::AttachmentReference::default().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
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
            &vk::RenderPassCreateInfo::default().attachments(&attachments).subpasses(&subpasses).dependencies(&deps),
            None,
        )?;

        // Descriptor set layout: one combined image sampler (set 0, binding 0).
        let bindings_dsl = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let set_layout = device
            .create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings_dsl), None)?;
        let pool_sizes =
            [vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1)];
        let desc_pool = device
            .create_descriptor_pool(&vk::DescriptorPoolCreateInfo::default().max_sets(1).pool_sizes(&pool_sizes), None)?;
        let set_layouts = [set_layout];
        let desc_set = device
            .allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default().descriptor_pool(desc_pool).set_layouts(&set_layouts),
            )?[0];

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
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert).name(c"main"),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag).name(c"main"),
        ];
        let bindings = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Instance>() as u32)
            .input_rate(vk::VertexInputRate::INSTANCE)];
        let attr = |loc: u32, format: vk::Format, offset: u32| {
            vk::VertexInputAttributeDescription::default().location(loc).binding(0).format(format).offset(offset)
        };
        let attrs = [
            attr(0, vk::Format::R32G32_SFLOAT, 0),
            attr(1, vk::Format::R32G32_SFLOAT, 8),
            attr(2, vk::Format::R32G32B32A32_SFLOAT, 16),
            attr(3, vk::Format::R32G32B32A32_SFLOAT, 32),
            attr(4, vk::Format::R32G32B32A32_SFLOAT, 48),
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
        let push_ranges =
            [vk::PushConstantRange::default().stage_flags(vk::ShaderStageFlags::VERTEX).offset(0).size(8)];
        let pipeline_layout = device.create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts).push_constant_ranges(&push_ranges),
            None,
        )?;
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
        let pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), &gpi, None).map_err(|(_, e)| e)?[0];
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
            let mt = find_memory_type(
                &mem_props,
                req.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let memory = device.allocate_memory(
                &vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt),
                None,
            )?;
            device.bind_buffer_memory(buffer, memory, 0)?;
            let mapped = device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())? as *mut Instance;
            frames.push(FrameData {
                fence: device
                    .create_fence(&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED), None)?,
                image_available: device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?,
                cmd,
                buffer,
                memory,
                mapped,
            });
        }

        // Atlas texture (RGBA8_UNORM), uploaded once through a staging buffer.
        let (atlas_image, atlas_memory, atlas_view) =
            Self::upload_atlas(&device, &mem_props, pool, queue, atlas_rgba)?;
        let sampler = device.create_sampler(
            &vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::NEAREST)
                .min_filter(vk::Filter::NEAREST)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE),
            None,
        )?;
        let image_info = [vk::DescriptorImageInfo::default()
            .sampler(sampler)
            .image_view(atlas_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        device.update_descriptor_sets(
            &[vk::WriteDescriptorSet::default()
                .dst_set(desc_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info)],
            &[],
        );

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
            can_capture,
            views: Vec::new(),
            images: Vec::new(),
            framebuffers: Vec::new(),
            render_finished: Vec::new(),
            render_pass,
            set_layout,
            desc_pool,
            desc_set,
            pipeline_layout,
            pipeline,
            pool,
            frames,
            capacity,
            frame: 0,
            dirty: false,
            mem_props,
            atlas_image,
            atlas_memory,
            atlas_view,
            sampler,
            gpu_name,
        };
        r.recreate_swapchain((window.inner_size().width, window.inner_size().height))?;
        Ok(r)
    }

    unsafe fn upload_atlas(
        device: &ash::Device,
        mem_props: &vk::PhysicalDeviceMemoryProperties,
        pool: vk::CommandPool,
        queue: vk::Queue,
        rgba: &[u8],
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), Box<dyn std::error::Error>> {
        let size = (ATLAS_SIZE * ATLAS_SIZE * 4) as u64;
        assert_eq!(rgba.len() as u64, size);

        let staging = device.create_buffer(
            &vk::BufferCreateInfo::default()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            None,
        )?;
        let req = device.get_buffer_memory_requirements(staging);
        let mt = find_memory_type(
            mem_props,
            req.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let staging_mem = device
            .allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt), None)?;
        device.bind_buffer_memory(staging, staging_mem, 0)?;
        let p = device.map_memory(staging_mem, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())? as *mut u8;
        std::ptr::copy_nonoverlapping(rgba.as_ptr(), p, rgba.len());
        device.unmap_memory(staging_mem);

        let image = device.create_image(
            &vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(vk::Extent3D { width: ATLAS_SIZE as u32, height: ATLAS_SIZE as u32, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED),
            None,
        )?;
        let req = device.get_image_memory_requirements(image);
        let mt = find_memory_type(mem_props, req.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
        let image_mem = device
            .allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt), None)?;
        device.bind_image_memory(image, image_mem, 0)?;

        let cmd = device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1),
        )?[0];
        device.begin_command_buffer(
            cmd,
            &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;
        let range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);
        let to_dst = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(range)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[to_dst],
        );
        let region = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default().aspect_mask(vk::ImageAspectFlags::COLOR).layer_count(1),
            )
            .image_extent(vk::Extent3D { width: ATLAS_SIZE as u32, height: ATLAS_SIZE as u32, depth: 1 });
        device.cmd_copy_buffer_to_image(cmd, staging, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
        let to_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(range)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[to_read],
        );
        device.end_command_buffer(cmd)?;
        let cmds = [cmd];
        device.queue_submit(queue, &[vk::SubmitInfo::default().command_buffers(&cmds)], vk::Fence::null())?;
        device.queue_wait_idle(queue)?;
        device.free_command_buffers(pool, &cmds);
        device.destroy_buffer(staging, None);
        device.free_memory(staging_mem, None);

        let view = device.create_image_view(
            &vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(range),
            None,
        )?;
        Ok((image, image_mem, view))
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
        self.images.clear();
        self.render_finished.clear();
    }

    /// Returns Ok(false) when the surface has zero size (minimised) and nothing was created.
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
        let mut usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        if self.can_capture {
            usage |= vk::ImageUsageFlags::TRANSFER_SRC;
        }
        let old = self.swapchain;
        let sci = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(count)
            .image_format(self.format.format)
            .image_color_space(self.format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(usage)
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
        self.images = self.swapchain_loader.get_swapchain_images(new)?;
        for &img in &self.images.clone() {
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
            self.render_finished.push(self.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?);
        }
        self.dirty = false;
        Ok(true)
    }

    /// Render one frame. `fill` writes instances into the mapped buffer and returns how many
    /// were written. `win_size` is the window size in pixels. With `capture`, the canvas area of
    /// this frame is copied back and returned.
    pub fn render<F: FnOnce(&mut [Instance]) -> usize>(
        &mut self,
        win_size: (u32, u32),
        capture: bool,
        fill: F,
    ) -> Result<Option<Captured>, String> {
        unsafe { self.render_inner(win_size, capture, fill).map_err(|e| e.to_string()) }
    }

    unsafe fn render_inner<F: FnOnce(&mut [Instance]) -> usize>(
        &mut self,
        win_size: (u32, u32),
        capture: bool,
        fill: F,
    ) -> Result<Option<Captured>, Box<dyn std::error::Error>> {
        if win_size.0 == 0 || win_size.1 == 0 {
            return Ok(None); // minimised
        }
        if capture && !self.can_capture {
            return Err("swapchain does not support TRANSFER_SRC".into());
        }
        if self.dirty
            || self.swapchain == vk::SwapchainKHR::null()
            || (win_size.0 != self.extent.width || win_size.1 != self.extent.height)
        {
            if !self.recreate_swapchain(win_size)? {
                return Ok(None);
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
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };
        self.device.reset_fences(&[fence])?;

        let f = &self.frames[fi];
        let slice = std::slice::from_raw_parts_mut(f.mapped, self.capacity);
        let n = fill(slice).min(self.capacity);

        let lb = Letterbox::fit(self.extent.width, self.extent.height);
        let cmd = f.cmd;
        self.device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())?;
        self.device.begin_command_buffer(
            cmd,
            &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;
        // The whole window is cleared black; the canvas draws into the letterboxed viewport.
        let clear = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] } }];
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
            &[vk::Viewport { x: lb.x, y: lb.y, width: lb.w, height: lb.h, min_depth: 0.0, max_depth: 1.0 }],
        );
        self.device.cmd_set_scissor(
            cmd,
            0,
            &[vk::Rect2D {
                offset: vk::Offset2D { x: lb.x as i32, y: lb.y as i32 },
                extent: vk::Extent2D { width: lb.w as u32, height: lb.h as u32 },
            }],
        );
        let canvas = [CANVAS_W, CANVAS_H];
        let canvas_bytes = std::slice::from_raw_parts(canvas.as_ptr() as *const u8, 8);
        self.device.cmd_push_constants(cmd, self.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, canvas_bytes);
        self.device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &[self.desc_set],
            &[],
        );
        self.device.cmd_bind_vertex_buffers(cmd, 0, &[f.buffer], &[0]);
        self.device.cmd_draw(cmd, 6, n as u32, 0, 0);
        self.device.cmd_end_render_pass(cmd);

        // Optional copy of the canvas area to a host buffer.
        let mut cap_buf: Option<(vk::Buffer, vk::DeviceMemory, usize)> = None;
        if capture {
            let size = lb.w as usize * lb.h as usize * 4;
            let buffer = self.device.create_buffer(
                &vk::BufferCreateInfo::default()
                    .size(size as u64)
                    .usage(vk::BufferUsageFlags::TRANSFER_DST)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE),
                None,
            )?;
            let req = self.device.get_buffer_memory_requirements(buffer);
            let mt = find_memory_type(
                &self.mem_props,
                req.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let memory = self.device.allocate_memory(
                &vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(mt),
                None,
            )?;
            self.device.bind_buffer_memory(buffer, memory, 0)?;
            cap_buf = Some((buffer, memory, size));

            let image = self.images[image_index as usize];
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1);
            let barrier = |old, new, src, dst| {
                vk::ImageMemoryBarrier::default()
                    .old_layout(old)
                    .new_layout(new)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresource_range(range)
                    .src_access_mask(src)
                    .dst_access_mask(dst)
            };
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier(
                    vk::ImageLayout::PRESENT_SRC_KHR,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::AccessFlags::MEMORY_WRITE,
                    vk::AccessFlags::TRANSFER_READ,
                )],
            );
            let region = vk::BufferImageCopy::default()
                .image_subresource(
                    vk::ImageSubresourceLayers::default().aspect_mask(vk::ImageAspectFlags::COLOR).layer_count(1),
                )
                .image_offset(vk::Offset3D { x: lb.x as i32, y: lb.y as i32, z: 0 })
                .image_extent(vk::Extent3D { width: lb.w as u32, height: lb.h as u32, depth: 1 });
            self.device.cmd_copy_image_to_buffer(
                cmd,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                buffer,
                &[region],
            );
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier(
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                    vk::AccessFlags::TRANSFER_READ,
                    vk::AccessFlags::empty(),
                )],
            );
        }
        self.device.end_command_buffer(cmd)?;

        let wait = [self.frames[fi].image_available];
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
        let present =
            vk::PresentInfoKHR::default().wait_semaphores(&signal).swapchains(&swapchains).image_indices(&indices);
        match self.swapchain_loader.queue_present(self.queue, &present) {
            Ok(sub) => {
                if sub || suboptimal {
                    self.dirty = true;
                }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => self.dirty = true,
            Err(e) => return Err(e.into()),
        }
        self.frame = (self.frame + 1) % FRAMES_IN_FLIGHT;

        let mut result = None;
        if let Some((buffer, memory, size)) = cap_buf {
            self.device.wait_for_fences(&[fence], true, u64::MAX)?;
            let p = self.device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())? as *const u8;
            let mut rgba = std::slice::from_raw_parts(p, size).to_vec();
            self.device.unmap_memory(memory);
            self.device.destroy_buffer(buffer, None);
            self.device.free_memory(memory, None);
            if matches!(self.format.format, vk::Format::B8G8R8A8_UNORM | vk::Format::B8G8R8A8_SRGB) {
                for px in rgba.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
            for px in rgba.chunks_exact_mut(4) {
                px[3] = 255;
            }
            result = Some(Captured { width: lb.w as u32, height: lb.h as u32, rgba });
        }
        Ok(result)
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
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.atlas_view, None);
            self.device.destroy_image(self.atlas_image, None);
            self.device.free_memory(self.atlas_memory, None);
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_pool(self.desc_pool, None);
            self.device.destroy_descriptor_set_layout(self.set_layout, None);
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
