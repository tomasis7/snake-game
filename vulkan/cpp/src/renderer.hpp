#pragma once
#include <vulkan/vulkan.h>

#include <cstdint>
#include <string>
#include <vector>

#include "atlas.hpp"
#include "instance.hpp"

struct GLFWwindow;

struct IRect {
    int x = 0, y = 0, w = 0, h = 0;
};

// A captured frame: RGBA8, cropped to the letterboxed canvas.
struct Capture {
    int width = 0, height = 0;
    std::vector<uint8_t> rgba;
};

class Renderer {
public:
    static constexpr int FRAMES_IN_FLIGHT = 2;

    // maxInstances: capacity of each per-frame instance buffer.
    // preferNoVsync: choose IMMEDIATE, else MAILBOX, else FIFO (benchmark); otherwise FIFO.
    Renderer(GLFWwindow* window, uint32_t maxInstances, bool preferNoVsync, const Atlas& atlas);
    ~Renderer();
    Renderer(const Renderer&) = delete;
    Renderer& operator=(const Renderer&) = delete;

    const std::string& gpuName() const { return gpuName_; }
    const char* presentModeName() const;
    // The largest centred 3:2 rect inside the framebuffer, in framebuffer pixels.
    IRect canvasRect() const;

    // Waits for this frame slot to be free and returns its mapped instance buffer
    // (capacity maxInstances), or nullptr if the window is minimised / unusable.
    Instance* beginFrame();
    // Draws `count` instances from the buffer returned by beginFrame and presents.
    // If `capture` is given, the canvas area is read back into it before presenting.
    void endFrame(uint32_t count, Capture* capture = nullptr);

private:
    struct Frame {
        VkCommandBuffer cmd{};
        VkSemaphore imageAvailable{};
        VkFence fence{};
        VkBuffer buffer{};
        VkDeviceMemory memory{};
        Instance* mapped{};
    };

    void createInstance();
    void pickDevice();
    void createDevice();
    void createSwapchain();
    void destroySwapchain(bool keepHandle);
    void createRenderPass();
    void createPipeline();
    void createFrames();
    void createAtlasTexture(const Atlas& atlas);
    void recreateSwapchain();
    void createFramebuffers();
    bool framebufferSizeValid() const;
    uint32_t findMemoryType(uint32_t bits, VkMemoryPropertyFlags props) const;
    void createBuffer(VkDeviceSize size, VkBufferUsageFlags usage, VkBuffer& buf, VkDeviceMemory& mem);

    GLFWwindow* window_;
    uint32_t maxInstances_;
    bool preferNoVsync_;

    VkInstance instance_{};
    VkDebugUtilsMessengerEXT messenger_{};
    VkSurfaceKHR surface_{};
    VkPhysicalDevice phys_{};
    VkDevice device_{};
    VkQueue queue_{};
    uint32_t queueFamily_ = 0;
    std::string gpuName_;

    VkSwapchainKHR swapchain_{};
    VkFormat format_{};
    VkExtent2D extent_{};
    VkPresentModeKHR presentMode_ = VK_PRESENT_MODE_FIFO_KHR;
    bool canCapture_ = false;
    std::vector<VkImage> images_;
    std::vector<VkImageView> views_;
    std::vector<VkFramebuffer> framebuffers_;
    std::vector<VkSemaphore> renderFinished_;  // one per swapchain image

    VkRenderPass renderPass_{};
    VkDescriptorSetLayout setLayout_{};
    VkPipelineLayout pipelineLayout_{};
    VkPipeline pipeline_{};
    VkCommandPool pool_{};
    Frame frames_[FRAMES_IN_FLIGHT]{};
    uint32_t frameIndex_ = 0;

    VkImage atlasImage_{};
    VkDeviceMemory atlasMemory_{};
    VkImageView atlasView_{};
    VkSampler sampler_{};
    VkDescriptorPool descPool_{};
    VkDescriptorSet descSet_{};
};
