#include "renderer.hpp"

#include <GLFW/glfw3.h>

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <stdexcept>

#define VK_CHECK(expr)                                                                   \
    do {                                                                                 \
        VkResult r_ = (expr);                                                            \
        if (r_ != VK_SUCCESS) {                                                          \
            std::fprintf(stderr, "Vulkan error %d at %s:%d (%s)\n", (int)r_, __FILE__,   \
                         __LINE__, #expr);                                               \
            std::exit(1);                                                                \
        }                                                                                \
    } while (0)

namespace {

std::vector<char> readFile(const std::string& path) {
    std::ifstream f(path, std::ios::binary | std::ios::ate);
    if (!f) throw std::runtime_error("cannot open " + path);
    size_t n = (size_t)f.tellg();
    std::vector<char> data(n);
    f.seekg(0);
    f.read(data.data(), (std::streamsize)n);
    return data;
}

#ifdef SNAKE_VALIDATION
VKAPI_ATTR VkBool32 VKAPI_CALL debugCb(VkDebugUtilsMessageSeverityFlagBitsEXT sev,
                                       VkDebugUtilsMessageTypeFlagsEXT, const VkDebugUtilsMessengerCallbackDataEXT* d,
                                       void*) {
    if (sev >= VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT)
        std::fprintf(stderr, "[validation] %s\n", d->pMessage);
    return VK_FALSE;
}
#endif

}  // namespace

Renderer::Renderer(GLFWwindow* window, uint32_t maxInstances, bool preferNoVsync, const Atlas& atlas)
    : window_(window), maxInstances_(maxInstances), preferNoVsync_(preferNoVsync) {
    createInstance();
    VK_CHECK(glfwCreateWindowSurface(instance_, window_, nullptr, &surface_));
    pickDevice();
    createDevice();
    createSwapchain();
    createRenderPass();
    createPipeline();
    createFrames();
    createAtlasTexture(atlas);
    createFramebuffers();  // need the render pass, so build them now
}

void Renderer::createFramebuffers() {
    for (size_t i = 0; i < views_.size(); ++i) {
        VkFramebufferCreateInfo fi{VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO};
        fi.renderPass = renderPass_;
        fi.attachmentCount = 1;
        fi.pAttachments = &views_[i];
        fi.width = extent_.width;
        fi.height = extent_.height;
        fi.layers = 1;
        VK_CHECK(vkCreateFramebuffer(device_, &fi, nullptr, &framebuffers_[i]));
    }
}

IRect Renderer::canvasRect() const {
    int w = (int)extent_.width, h = (int)extent_.height;
    IRect r;
    if (w * 2 > h * 3) {  // wider than 3:2: pillarbox
        r.h = h;
        r.w = h * 3 / 2;
    } else {
        r.w = w;
        r.h = w * 2 / 3;
    }
    r.x = (w - r.w) / 2;
    r.y = (h - r.h) / 2;
    return r;
}

Renderer::~Renderer() {
    vkDeviceWaitIdle(device_);
    for (auto& f : frames_) {
        vkDestroyBuffer(device_, f.buffer, nullptr);
        vkUnmapMemory(device_, f.memory);
        vkFreeMemory(device_, f.memory, nullptr);
        vkDestroySemaphore(device_, f.imageAvailable, nullptr);
        vkDestroyFence(device_, f.fence, nullptr);
    }
    vkDestroyCommandPool(device_, pool_, nullptr);
    vkDestroyDescriptorPool(device_, descPool_, nullptr);
    vkDestroySampler(device_, sampler_, nullptr);
    vkDestroyImageView(device_, atlasView_, nullptr);
    vkDestroyImage(device_, atlasImage_, nullptr);
    vkFreeMemory(device_, atlasMemory_, nullptr);
    destroySwapchain(false);
    vkDestroyPipeline(device_, pipeline_, nullptr);
    vkDestroyPipelineLayout(device_, pipelineLayout_, nullptr);
    vkDestroyDescriptorSetLayout(device_, setLayout_, nullptr);
    vkDestroyRenderPass(device_, renderPass_, nullptr);
    vkDestroyDevice(device_, nullptr);
    vkDestroySurfaceKHR(instance_, surface_, nullptr);
#ifdef SNAKE_VALIDATION
    auto destroy = (PFN_vkDestroyDebugUtilsMessengerEXT)vkGetInstanceProcAddr(
        instance_, "vkDestroyDebugUtilsMessengerEXT");
    if (destroy && messenger_) destroy(instance_, messenger_, nullptr);
#endif
    vkDestroyInstance(instance_, nullptr);
}

const char* Renderer::presentModeName() const {
    switch (presentMode_) {
        case VK_PRESENT_MODE_IMMEDIATE_KHR: return "IMMEDIATE";
        case VK_PRESENT_MODE_MAILBOX_KHR: return "MAILBOX";
        case VK_PRESENT_MODE_FIFO_KHR: return "FIFO";
        case VK_PRESENT_MODE_FIFO_RELAXED_KHR: return "FIFO_RELAXED";
        default: return "OTHER";
    }
}

void Renderer::createInstance() {
    VkApplicationInfo app{VK_STRUCTURE_TYPE_APPLICATION_INFO};
    app.pApplicationName = "Vulkan Snake (C++)";
    app.apiVersion = VK_API_VERSION_1_1;

    uint32_t n = 0;
    const char** glfwExt = glfwGetRequiredInstanceExtensions(&n);
    if (!glfwExt) throw std::runtime_error("GLFW: Vulkan not available");
    std::vector<const char*> exts(glfwExt, glfwExt + n);
    std::vector<const char*> layers;

#ifdef SNAKE_VALIDATION
    {
        uint32_t lc = 0;
        vkEnumerateInstanceLayerProperties(&lc, nullptr);
        std::vector<VkLayerProperties> props(lc);
        vkEnumerateInstanceLayerProperties(&lc, props.data());
        for (auto& p : props)
            if (std::strcmp(p.layerName, "VK_LAYER_KHRONOS_validation") == 0)
                layers.push_back("VK_LAYER_KHRONOS_validation");
        if (layers.empty()) std::fprintf(stderr, "validation layer not available\n");
        exts.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
    }
#endif

    VkInstanceCreateInfo ci{VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO};
    ci.pApplicationInfo = &app;
    ci.enabledExtensionCount = (uint32_t)exts.size();
    ci.ppEnabledExtensionNames = exts.data();
    ci.enabledLayerCount = (uint32_t)layers.size();
    ci.ppEnabledLayerNames = layers.data();

#ifdef SNAKE_VALIDATION
    VkDebugUtilsMessengerCreateInfoEXT mi{VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT};
    mi.messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
                         VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;
    mi.messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT |
                     VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                     VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
    mi.pfnUserCallback = debugCb;
    ci.pNext = &mi;  // also covers instance create/destroy
#endif

    VK_CHECK(vkCreateInstance(&ci, nullptr, &instance_));

#ifdef SNAKE_VALIDATION
    auto create = (PFN_vkCreateDebugUtilsMessengerEXT)vkGetInstanceProcAddr(
        instance_, "vkCreateDebugUtilsMessengerEXT");
    if (create) create(instance_, &mi, nullptr, &messenger_);
#endif
}

void Renderer::pickDevice() {
    uint32_t n = 0;
    vkEnumeratePhysicalDevices(instance_, &n, nullptr);
    std::vector<VkPhysicalDevice> devs(n);
    vkEnumeratePhysicalDevices(instance_, &n, devs.data());

    int bestRank = -1;
    for (auto d : devs) {
        VkPhysicalDeviceProperties p;
        vkGetPhysicalDeviceProperties(d, &p);
        int rank;
        switch (p.deviceType) {
            case VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU: rank = 3; break;
            case VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU: rank = 2; break;
            case VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU: rank = 1; break;
            default: continue;  // CPU / other (llvmpipe)
        }
        if (std::strstr(p.deviceName, "llvmpipe")) continue;

        uint32_t qn = 0;
        vkGetPhysicalDeviceQueueFamilyProperties(d, &qn, nullptr);
        std::vector<VkQueueFamilyProperties> qf(qn);
        vkGetPhysicalDeviceQueueFamilyProperties(d, &qn, qf.data());
        for (uint32_t i = 0; i < qn; ++i) {
            VkBool32 present = VK_FALSE;
            vkGetPhysicalDeviceSurfaceSupportKHR(d, i, surface_, &present);
            if ((qf[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) && present && rank > bestRank) {
                bestRank = rank;
                phys_ = d;
                queueFamily_ = i;
                gpuName_ = p.deviceName;
                break;
            }
        }
    }
    if (!phys_) throw std::runtime_error("no suitable GPU found");
}

void Renderer::createDevice() {
    float prio = 1.0f;
    VkDeviceQueueCreateInfo qi{VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO};
    qi.queueFamilyIndex = queueFamily_;
    qi.queueCount = 1;
    qi.pQueuePriorities = &prio;
    const char* ext = VK_KHR_SWAPCHAIN_EXTENSION_NAME;
    VkDeviceCreateInfo ci{VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO};
    ci.queueCreateInfoCount = 1;
    ci.pQueueCreateInfos = &qi;
    ci.enabledExtensionCount = 1;
    ci.ppEnabledExtensionNames = &ext;
    VK_CHECK(vkCreateDevice(phys_, &ci, nullptr, &device_));
    vkGetDeviceQueue(device_, queueFamily_, 0, &queue_);
}

bool Renderer::framebufferSizeValid() const {
    int w = 0, h = 0;
    glfwGetFramebufferSize(window_, &w, &h);
    return w > 0 && h > 0;
}

void Renderer::createSwapchain() {
    VkSurfaceCapabilitiesKHR caps;
    VK_CHECK(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(phys_, surface_, &caps));

    uint32_t n = 0;
    vkGetPhysicalDeviceSurfaceFormatsKHR(phys_, surface_, &n, nullptr);
    std::vector<VkSurfaceFormatKHR> fmts(n);
    vkGetPhysicalDeviceSurfaceFormatsKHR(phys_, surface_, &n, fmts.data());
    VkSurfaceFormatKHR chosen = fmts[0];
    for (auto& f : fmts)
        if (f.format == VK_FORMAT_B8G8R8A8_UNORM && f.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR)
            chosen = f;
    format_ = chosen.format;

    vkGetPhysicalDeviceSurfacePresentModesKHR(phys_, surface_, &n, nullptr);
    std::vector<VkPresentModeKHR> modes(n);
    vkGetPhysicalDeviceSurfacePresentModesKHR(phys_, surface_, &n, modes.data());
    auto has = [&](VkPresentModeKHR m) { return std::find(modes.begin(), modes.end(), m) != modes.end(); };
    presentMode_ = VK_PRESENT_MODE_FIFO_KHR;
    if (preferNoVsync_) {
        if (has(VK_PRESENT_MODE_IMMEDIATE_KHR)) presentMode_ = VK_PRESENT_MODE_IMMEDIATE_KHR;
        else if (has(VK_PRESENT_MODE_MAILBOX_KHR)) presentMode_ = VK_PRESENT_MODE_MAILBOX_KHR;
    }

    if (caps.currentExtent.width != 0xFFFFFFFFu) {
        extent_ = caps.currentExtent;
    } else {
        int w, h;
        glfwGetFramebufferSize(window_, &w, &h);
        extent_.width = std::clamp((uint32_t)w, caps.minImageExtent.width, caps.maxImageExtent.width);
        extent_.height = std::clamp((uint32_t)h, caps.minImageExtent.height, caps.maxImageExtent.height);
    }

    uint32_t count = caps.minImageCount + 1;
    if (caps.maxImageCount > 0) count = std::min(count, caps.maxImageCount);

    VkCompositeAlphaFlagBitsKHR alpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
    if (!(caps.supportedCompositeAlpha & alpha)) {
        for (uint32_t b = 1; b <= VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR; b <<= 1)
            if (caps.supportedCompositeAlpha & b) { alpha = (VkCompositeAlphaFlagBitsKHR)b; break; }
    }

    VkSwapchainKHR old = swapchain_;
    VkSwapchainCreateInfoKHR ci{VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR};
    ci.surface = surface_;
    ci.minImageCount = count;
    ci.imageFormat = format_;
    ci.imageColorSpace = chosen.colorSpace;
    ci.imageExtent = extent_;
    ci.imageArrayLayers = 1;
    canCapture_ = (caps.supportedUsageFlags & VK_IMAGE_USAGE_TRANSFER_SRC_BIT) != 0;
    ci.imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | (canCapture_ ? VK_IMAGE_USAGE_TRANSFER_SRC_BIT : 0);
    ci.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;
    ci.preTransform = caps.currentTransform;
    ci.compositeAlpha = alpha;
    ci.presentMode = presentMode_;
    ci.clipped = VK_TRUE;
    ci.oldSwapchain = old;
    VK_CHECK(vkCreateSwapchainKHR(device_, &ci, nullptr, &swapchain_));
    if (old) vkDestroySwapchainKHR(device_, old, nullptr);

    vkGetSwapchainImagesKHR(device_, swapchain_, &n, nullptr);
    images_.resize(n);
    vkGetSwapchainImagesKHR(device_, swapchain_, &n, images_.data());

    views_.resize(n);
    for (uint32_t i = 0; i < n; ++i) {
        VkImageViewCreateInfo vi{VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO};
        vi.image = images_[i];
        vi.viewType = VK_IMAGE_VIEW_TYPE_2D;
        vi.format = format_;
        vi.subresourceRange = {VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1};
        VK_CHECK(vkCreateImageView(device_, &vi, nullptr, &views_[i]));
    }

    framebuffers_.assign(n, VK_NULL_HANDLE);
    renderFinished_.resize(n);
    for (auto& s : renderFinished_) {
        VkSemaphoreCreateInfo si{VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO};
        VK_CHECK(vkCreateSemaphore(device_, &si, nullptr, &s));
    }
}

// Destroys per-image resources. If keepHandle, the swapchain itself is kept to be
// passed as oldSwapchain on recreation.
void Renderer::destroySwapchain(bool keepHandle) {
    for (auto fb : framebuffers_) vkDestroyFramebuffer(device_, fb, nullptr);
    for (auto v : views_) vkDestroyImageView(device_, v, nullptr);
    for (auto s : renderFinished_) vkDestroySemaphore(device_, s, nullptr);
    framebuffers_.clear();
    views_.clear();
    renderFinished_.clear();
    images_.clear();
    if (!keepHandle && swapchain_) {
        vkDestroySwapchainKHR(device_, swapchain_, nullptr);
        swapchain_ = VK_NULL_HANDLE;
    }
}

void Renderer::recreateSwapchain() {
    vkDeviceWaitIdle(device_);
    if (!framebufferSizeValid()) return;
    destroySwapchain(true);
    createSwapchain();
    createFramebuffers();
}

void Renderer::createRenderPass() {
    VkAttachmentDescription att{};
    att.format = format_;
    att.samples = VK_SAMPLE_COUNT_1_BIT;
    att.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
    att.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
    att.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
    att.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
    att.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    att.finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;
    VkAttachmentReference ref{0, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL};
    VkSubpassDescription sub{};
    sub.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
    sub.colorAttachmentCount = 1;
    sub.pColorAttachments = &ref;
    VkSubpassDependency dep{};
    dep.srcSubpass = VK_SUBPASS_EXTERNAL;
    dep.dstSubpass = 0;
    dep.srcStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    dep.dstStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    dep.srcAccessMask = 0;
    dep.dstAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT;
    VkRenderPassCreateInfo ci{VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO};
    ci.attachmentCount = 1;
    ci.pAttachments = &att;
    ci.subpassCount = 1;
    ci.pSubpasses = &sub;
    ci.dependencyCount = 1;
    ci.pDependencies = &dep;
    VK_CHECK(vkCreateRenderPass(device_, &ci, nullptr, &renderPass_));
}

void Renderer::createPipeline() {
    auto load = [&](const char* name) {
        auto code = readFile(std::string(SPV_DIR) + "/" + name);
        VkShaderModuleCreateInfo ci{VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO};
        ci.codeSize = code.size();
        ci.pCode = reinterpret_cast<const uint32_t*>(code.data());
        VkShaderModule m;
        VK_CHECK(vkCreateShaderModule(device_, &ci, nullptr, &m));
        return m;
    };
    VkShaderModule vs = load("quad.vert.spv");
    VkShaderModule fs = load("quad.frag.spv");

    VkPipelineShaderStageCreateInfo stages[2]{};
    stages[0].sType = stages[1].sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    stages[0].stage = VK_SHADER_STAGE_VERTEX_BIT;
    stages[0].module = vs;
    stages[0].pName = "main";
    stages[1].stage = VK_SHADER_STAGE_FRAGMENT_BIT;
    stages[1].module = fs;
    stages[1].pName = "main";

    VkVertexInputBindingDescription bind{0, sizeof(Instance), VK_VERTEX_INPUT_RATE_INSTANCE};
    VkVertexInputAttributeDescription attrs[5] = {
        {0, 0, VK_FORMAT_R32G32_SFLOAT, offsetof(Instance, pos)},
        {1, 0, VK_FORMAT_R32G32_SFLOAT, offsetof(Instance, size)},
        {2, 0, VK_FORMAT_R32G32B32A32_SFLOAT, offsetof(Instance, color)},
        {3, 0, VK_FORMAT_R32G32B32A32_SFLOAT, offsetof(Instance, uv)},
        {4, 0, VK_FORMAT_R32G32B32A32_SFLOAT, offsetof(Instance, params)},
    };
    VkPipelineVertexInputStateCreateInfo vin{VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO};
    vin.vertexBindingDescriptionCount = 1;
    vin.pVertexBindingDescriptions = &bind;
    vin.vertexAttributeDescriptionCount = 5;
    vin.pVertexAttributeDescriptions = attrs;

    VkPipelineInputAssemblyStateCreateInfo ia{VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO};
    ia.topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;

    VkPipelineViewportStateCreateInfo vp{VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO};
    vp.viewportCount = 1;
    vp.scissorCount = 1;

    VkPipelineRasterizationStateCreateInfo rs{VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO};
    rs.polygonMode = VK_POLYGON_MODE_FILL;
    rs.cullMode = VK_CULL_MODE_NONE;
    rs.frontFace = VK_FRONT_FACE_CLOCKWISE;
    rs.lineWidth = 1.0f;

    VkPipelineMultisampleStateCreateInfo ms{VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO};
    ms.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;

    VkPipelineColorBlendAttachmentState cba{};
    cba.blendEnable = VK_TRUE;
    cba.srcColorBlendFactor = VK_BLEND_FACTOR_SRC_ALPHA;
    cba.dstColorBlendFactor = VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
    cba.colorBlendOp = VK_BLEND_OP_ADD;
    cba.srcAlphaBlendFactor = VK_BLEND_FACTOR_ONE;
    cba.dstAlphaBlendFactor = VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
    cba.alphaBlendOp = VK_BLEND_OP_ADD;
    cba.colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT |
                         VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
    VkPipelineColorBlendStateCreateInfo cb{VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO};
    cb.attachmentCount = 1;
    cb.pAttachments = &cba;

    VkDynamicState dyn[2] = {VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR};
    VkPipelineDynamicStateCreateInfo ds{VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO};
    ds.dynamicStateCount = 2;
    ds.pDynamicStates = dyn;

    VkDescriptorSetLayoutBinding dsb{};
    dsb.binding = 0;
    dsb.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
    dsb.descriptorCount = 1;
    dsb.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT;
    VkDescriptorSetLayoutCreateInfo dli{VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO};
    dli.bindingCount = 1;
    dli.pBindings = &dsb;
    VK_CHECK(vkCreateDescriptorSetLayout(device_, &dli, nullptr, &setLayout_));

    VkPushConstantRange pcr{VK_SHADER_STAGE_VERTEX_BIT, 0, 2 * sizeof(float)};
    VkPipelineLayoutCreateInfo li{VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO};
    li.setLayoutCount = 1;
    li.pSetLayouts = &setLayout_;
    li.pushConstantRangeCount = 1;
    li.pPushConstantRanges = &pcr;
    VK_CHECK(vkCreatePipelineLayout(device_, &li, nullptr, &pipelineLayout_));

    VkGraphicsPipelineCreateInfo pi{VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO};
    pi.stageCount = 2;
    pi.pStages = stages;
    pi.pVertexInputState = &vin;
    pi.pInputAssemblyState = &ia;
    pi.pViewportState = &vp;
    pi.pRasterizationState = &rs;
    pi.pMultisampleState = &ms;
    pi.pColorBlendState = &cb;
    pi.pDynamicState = &ds;
    pi.layout = pipelineLayout_;
    pi.renderPass = renderPass_;
    VK_CHECK(vkCreateGraphicsPipelines(device_, VK_NULL_HANDLE, 1, &pi, nullptr, &pipeline_));

    vkDestroyShaderModule(device_, vs, nullptr);
    vkDestroyShaderModule(device_, fs, nullptr);
}

uint32_t Renderer::findMemoryType(uint32_t bits, VkMemoryPropertyFlags props) const {
    VkPhysicalDeviceMemoryProperties mp;
    vkGetPhysicalDeviceMemoryProperties(phys_, &mp);
    for (uint32_t i = 0; i < mp.memoryTypeCount; ++i)
        if ((bits & (1u << i)) && (mp.memoryTypes[i].propertyFlags & props) == props) return i;
    throw std::runtime_error("no suitable memory type");
}

void Renderer::createFrames() {
    VkCommandPoolCreateInfo pi{VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO};
    pi.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
    pi.queueFamilyIndex = queueFamily_;
    VK_CHECK(vkCreateCommandPool(device_, &pi, nullptr, &pool_));

    for (auto& f : frames_) {
        VkCommandBufferAllocateInfo ai{VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO};
        ai.commandPool = pool_;
        ai.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
        ai.commandBufferCount = 1;
        VK_CHECK(vkAllocateCommandBuffers(device_, &ai, &f.cmd));

        VkSemaphoreCreateInfo si{VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO};
        VK_CHECK(vkCreateSemaphore(device_, &si, nullptr, &f.imageAvailable));
        VkFenceCreateInfo fi{VK_STRUCTURE_TYPE_FENCE_CREATE_INFO};
        fi.flags = VK_FENCE_CREATE_SIGNALED_BIT;
        VK_CHECK(vkCreateFence(device_, &fi, nullptr, &f.fence));

        VkBufferCreateInfo bi{VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO};
        bi.size = (VkDeviceSize)maxInstances_ * sizeof(Instance);
        bi.usage = VK_BUFFER_USAGE_VERTEX_BUFFER_BIT;
        bi.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
        VK_CHECK(vkCreateBuffer(device_, &bi, nullptr, &f.buffer));
        VkMemoryRequirements mr;
        vkGetBufferMemoryRequirements(device_, f.buffer, &mr);
        VkMemoryAllocateInfo mi{VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO};
        mi.allocationSize = mr.size;
        mi.memoryTypeIndex = findMemoryType(
            mr.memoryTypeBits, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);
        VK_CHECK(vkAllocateMemory(device_, &mi, nullptr, &f.memory));
        VK_CHECK(vkBindBufferMemory(device_, f.buffer, f.memory, 0));
        void* p = nullptr;
        VK_CHECK(vkMapMemory(device_, f.memory, 0, VK_WHOLE_SIZE, 0, &p));
        f.mapped = static_cast<Instance*>(p);
    }
}

Instance* Renderer::beginFrame() {
    if (!framebufferSizeValid()) return nullptr;
    int w, h;
    glfwGetFramebufferSize(window_, &w, &h);
    if ((uint32_t)w != extent_.width || (uint32_t)h != extent_.height) {
        recreateSwapchain();
        if (framebuffers_.empty()) return nullptr;
    }
    Frame& f = frames_[frameIndex_];
    VK_CHECK(vkWaitForFences(device_, 1, &f.fence, VK_TRUE, UINT64_MAX));
    return f.mapped;
}

void Renderer::endFrame(uint32_t count, Capture* capture) {
    Frame& f = frames_[frameIndex_];
    uint32_t img = 0;
    VkResult r = vkAcquireNextImageKHR(device_, swapchain_, UINT64_MAX, f.imageAvailable,
                                       VK_NULL_HANDLE, &img);
    if (r == VK_ERROR_OUT_OF_DATE_KHR) {
        recreateSwapchain();
        return;
    }
    if (r != VK_SUCCESS && r != VK_SUBOPTIMAL_KHR) VK_CHECK(r);
    bool suboptimal = (r == VK_SUBOPTIMAL_KHR);

    VK_CHECK(vkResetFences(device_, 1, &f.fence));
    VK_CHECK(vkResetCommandBuffer(f.cmd, 0));
    VkCommandBufferBeginInfo bi{VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO};
    bi.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    VK_CHECK(vkBeginCommandBuffer(f.cmd, &bi));

    VkClearValue clear{};
    clear.color = {{0.0f, 0.0f, 0.0f, 1.0f}};
    VkRenderPassBeginInfo rp{VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO};
    rp.renderPass = renderPass_;
    rp.framebuffer = framebuffers_[img];
    rp.renderArea = {{0, 0}, extent_};
    rp.clearValueCount = 1;
    rp.pClearValues = &clear;
    vkCmdBeginRenderPass(f.cmd, &rp, VK_SUBPASS_CONTENTS_INLINE);
    vkCmdBindPipeline(f.cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline_);
    // Letterbox: the 1200 x 800 canvas fills the largest centred 3:2 rect.
    IRect cr = canvasRect();
    VkViewport vp{(float)cr.x, (float)cr.y, (float)cr.w, (float)cr.h, 0.0f, 1.0f};
    VkRect2D sc{{cr.x, cr.y}, {(uint32_t)cr.w, (uint32_t)cr.h}};
    vkCmdSetViewport(f.cmd, 0, 1, &vp);
    vkCmdSetScissor(f.cmd, 0, 1, &sc);
    float canvasSize[2] = {1200.0f, 800.0f};
    vkCmdPushConstants(f.cmd, pipelineLayout_, VK_SHADER_STAGE_VERTEX_BIT, 0, sizeof canvasSize, canvasSize);
    vkCmdBindDescriptorSets(f.cmd, VK_PIPELINE_BIND_POINT_GRAPHICS, pipelineLayout_, 0, 1, &descSet_, 0, nullptr);
    VkDeviceSize off = 0;
    vkCmdBindVertexBuffers(f.cmd, 0, 1, &f.buffer, &off);
    vkCmdDraw(f.cmd, 6, std::min(count, maxInstances_), 0, 0);
    vkCmdEndRenderPass(f.cmd);

    VkBuffer capBuf{};
    VkDeviceMemory capMem{};
    if (capture && canCapture_) {
        createBuffer((VkDeviceSize)cr.w * cr.h * 4, VK_BUFFER_USAGE_TRANSFER_DST_BIT, capBuf, capMem);
        VkImageMemoryBarrier b{VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER};
        b.srcAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT;
        b.dstAccessMask = VK_ACCESS_TRANSFER_READ_BIT;
        b.oldLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;
        b.newLayout = VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL;
        b.srcQueueFamilyIndex = b.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        b.image = images_[img];
        b.subresourceRange = {VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1};
        vkCmdPipelineBarrier(f.cmd, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, 0,
                             0, nullptr, 0, nullptr, 1, &b);
        VkBufferImageCopy region{};
        region.imageSubresource = {VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1};
        region.imageOffset = {cr.x, cr.y, 0};
        region.imageExtent = {(uint32_t)cr.w, (uint32_t)cr.h, 1};
        vkCmdCopyImageToBuffer(f.cmd, images_[img], VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, capBuf, 1, &region);
        b.srcAccessMask = VK_ACCESS_TRANSFER_READ_BIT;
        b.dstAccessMask = 0;
        b.oldLayout = VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL;
        b.newLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;
        vkCmdPipelineBarrier(f.cmd, VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, 0, 0,
                             nullptr, 0, nullptr, 1, &b);
    }
    VK_CHECK(vkEndCommandBuffer(f.cmd));

    VkPipelineStageFlags wait = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    VkSubmitInfo si{VK_STRUCTURE_TYPE_SUBMIT_INFO};
    si.waitSemaphoreCount = 1;
    si.pWaitSemaphores = &f.imageAvailable;
    si.pWaitDstStageMask = &wait;
    si.commandBufferCount = 1;
    si.pCommandBuffers = &f.cmd;
    si.signalSemaphoreCount = 1;
    si.pSignalSemaphores = &renderFinished_[img];
    VK_CHECK(vkQueueSubmit(queue_, 1, &si, f.fence));

    if (capBuf) {
        VK_CHECK(vkWaitForFences(device_, 1, &f.fence, VK_TRUE, UINT64_MAX));
        void* p = nullptr;
        VK_CHECK(vkMapMemory(device_, capMem, 0, VK_WHOLE_SIZE, 0, &p));
        capture->width = cr.w;
        capture->height = cr.h;
        capture->rgba.resize((size_t)cr.w * cr.h * 4);
        std::memcpy(capture->rgba.data(), p, capture->rgba.size());
        vkUnmapMemory(device_, capMem);
        vkDestroyBuffer(device_, capBuf, nullptr);
        vkFreeMemory(device_, capMem, nullptr);
        if (format_ == VK_FORMAT_B8G8R8A8_UNORM || format_ == VK_FORMAT_B8G8R8A8_SRGB)
            for (size_t i = 0; i < capture->rgba.size(); i += 4) std::swap(capture->rgba[i], capture->rgba[i + 2]);
        for (size_t i = 3; i < capture->rgba.size(); i += 4) capture->rgba[i] = 255;
    }

    VkPresentInfoKHR pi{VK_STRUCTURE_TYPE_PRESENT_INFO_KHR};
    pi.waitSemaphoreCount = 1;
    pi.pWaitSemaphores = &renderFinished_[img];
    pi.swapchainCount = 1;
    pi.pSwapchains = &swapchain_;
    pi.pImageIndices = &img;
    r = vkQueuePresentKHR(queue_, &pi);
    frameIndex_ = (frameIndex_ + 1) % FRAMES_IN_FLIGHT;
    if (r == VK_ERROR_OUT_OF_DATE_KHR || r == VK_SUBOPTIMAL_KHR || suboptimal) {
        recreateSwapchain();
    } else {
        VK_CHECK(r);
    }
}

void Renderer::createBuffer(VkDeviceSize size, VkBufferUsageFlags usage, VkBuffer& buf, VkDeviceMemory& mem) {
    VkBufferCreateInfo bi{VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO};
    bi.size = size;
    bi.usage = usage;
    bi.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    VK_CHECK(vkCreateBuffer(device_, &bi, nullptr, &buf));
    VkMemoryRequirements mr;
    vkGetBufferMemoryRequirements(device_, buf, &mr);
    VkMemoryAllocateInfo mi{VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO};
    mi.allocationSize = mr.size;
    mi.memoryTypeIndex = findMemoryType(mr.memoryTypeBits,
                                        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);
    VK_CHECK(vkAllocateMemory(device_, &mi, nullptr, &mem));
    VK_CHECK(vkBindBufferMemory(device_, buf, mem, 0));
}

// Uploads the CPU atlas once through a staging buffer and binds it as set 0, binding 0.
void Renderer::createAtlasTexture(const Atlas& atlas) {
    VkDeviceSize bytes = (VkDeviceSize)atlas.width * atlas.height * 4;
    VkBuffer staging{};
    VkDeviceMemory stagingMem{};
    createBuffer(bytes, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, staging, stagingMem);
    void* p = nullptr;
    VK_CHECK(vkMapMemory(device_, stagingMem, 0, VK_WHOLE_SIZE, 0, &p));
    std::memcpy(p, atlas.rgba.data(), (size_t)bytes);
    vkUnmapMemory(device_, stagingMem);

    VkImageCreateInfo ii{VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO};
    ii.imageType = VK_IMAGE_TYPE_2D;
    ii.format = VK_FORMAT_R8G8B8A8_UNORM;
    ii.extent = {(uint32_t)atlas.width, (uint32_t)atlas.height, 1};
    ii.mipLevels = 1;
    ii.arrayLayers = 1;
    ii.samples = VK_SAMPLE_COUNT_1_BIT;
    ii.tiling = VK_IMAGE_TILING_OPTIMAL;
    ii.usage = VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT;
    ii.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    VK_CHECK(vkCreateImage(device_, &ii, nullptr, &atlasImage_));
    VkMemoryRequirements mr;
    vkGetImageMemoryRequirements(device_, atlasImage_, &mr);
    VkMemoryAllocateInfo mi{VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO};
    mi.allocationSize = mr.size;
    mi.memoryTypeIndex = findMemoryType(mr.memoryTypeBits, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    VK_CHECK(vkAllocateMemory(device_, &mi, nullptr, &atlasMemory_));
    VK_CHECK(vkBindImageMemory(device_, atlasImage_, atlasMemory_, 0));

    VkCommandBufferAllocateInfo ai{VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO};
    ai.commandPool = pool_;
    ai.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    ai.commandBufferCount = 1;
    VkCommandBuffer cmd;
    VK_CHECK(vkAllocateCommandBuffers(device_, &ai, &cmd));
    VkCommandBufferBeginInfo bi{VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO};
    bi.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    VK_CHECK(vkBeginCommandBuffer(cmd, &bi));

    VkImageMemoryBarrier b{VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER};
    b.srcQueueFamilyIndex = b.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
    b.image = atlasImage_;
    b.subresourceRange = {VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1};
    b.oldLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    b.newLayout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;
    b.dstAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT;
    vkCmdPipelineBarrier(cmd, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, 0, 0, nullptr, 0,
                         nullptr, 1, &b);
    VkBufferImageCopy region{};
    region.imageSubresource = {VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1};
    region.imageExtent = {(uint32_t)atlas.width, (uint32_t)atlas.height, 1};
    vkCmdCopyBufferToImage(cmd, staging, atlasImage_, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &region);
    b.oldLayout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;
    b.newLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    b.srcAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT;
    b.dstAccessMask = VK_ACCESS_SHADER_READ_BIT;
    vkCmdPipelineBarrier(cmd, VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, 0, 0, nullptr, 0,
                         nullptr, 1, &b);
    VK_CHECK(vkEndCommandBuffer(cmd));
    VkSubmitInfo si{VK_STRUCTURE_TYPE_SUBMIT_INFO};
    si.commandBufferCount = 1;
    si.pCommandBuffers = &cmd;
    VK_CHECK(vkQueueSubmit(queue_, 1, &si, VK_NULL_HANDLE));
    VK_CHECK(vkQueueWaitIdle(queue_));
    vkFreeCommandBuffers(device_, pool_, 1, &cmd);
    vkDestroyBuffer(device_, staging, nullptr);
    vkFreeMemory(device_, stagingMem, nullptr);

    VkImageViewCreateInfo vi{VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO};
    vi.image = atlasImage_;
    vi.viewType = VK_IMAGE_VIEW_TYPE_2D;
    vi.format = VK_FORMAT_R8G8B8A8_UNORM;
    vi.subresourceRange = {VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1};
    VK_CHECK(vkCreateImageView(device_, &vi, nullptr, &atlasView_));

    VkSamplerCreateInfo sci{VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO};
    sci.magFilter = sci.minFilter = VK_FILTER_NEAREST;
    sci.mipmapMode = VK_SAMPLER_MIPMAP_MODE_NEAREST;
    sci.addressModeU = sci.addressModeV = sci.addressModeW = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    sci.maxLod = 0.0f;
    VK_CHECK(vkCreateSampler(device_, &sci, nullptr, &sampler_));

    VkDescriptorPoolSize ps{VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, 1};
    VkDescriptorPoolCreateInfo dpi{VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO};
    dpi.maxSets = 1;
    dpi.poolSizeCount = 1;
    dpi.pPoolSizes = &ps;
    VK_CHECK(vkCreateDescriptorPool(device_, &dpi, nullptr, &descPool_));
    VkDescriptorSetAllocateInfo dai{VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO};
    dai.descriptorPool = descPool_;
    dai.descriptorSetCount = 1;
    dai.pSetLayouts = &setLayout_;
    VK_CHECK(vkAllocateDescriptorSets(device_, &dai, &descSet_));
    VkDescriptorImageInfo dii{sampler_, atlasView_, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL};
    VkWriteDescriptorSet w{VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET};
    w.dstSet = descSet_;
    w.dstBinding = 0;
    w.descriptorCount = 1;
    w.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
    w.pImageInfo = &dii;
    vkUpdateDescriptorSets(device_, 1, &w, 0, nullptr);
}
