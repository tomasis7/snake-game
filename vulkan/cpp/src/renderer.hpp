#pragma once
#include <vulkan/vulkan.h>

#include <cstdint>
#include <string>
#include <vector>

struct GLFWwindow;

struct Instance {
    float pos[2];
    float size[2];
    float color[4];
};
static_assert(sizeof(Instance) == 32);

class Renderer {
public:
    static constexpr int FRAMES_IN_FLIGHT = 2;

    // maxInstances: capacity of each per-frame instance buffer.
    // preferNoVsync: choose IMMEDIATE, else MAILBOX, else FIFO (benchmark); otherwise FIFO.
    Renderer(GLFWwindow* window, uint32_t maxInstances, bool preferNoVsync);
    ~Renderer();
    Renderer(const Renderer&) = delete;
    Renderer& operator=(const Renderer&) = delete;

    const std::string& gpuName() const { return gpuName_; }
    const char* presentModeName() const;

    // Waits for this frame slot to be free and returns its mapped instance buffer
    // (capacity maxInstances), or nullptr if the window is minimised / unusable.
    Instance* beginFrame();
    // Draws `count` instances from the buffer returned by beginFrame and presents.
    void endFrame(uint32_t count);

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
    void recreateSwapchain();
    bool framebufferSizeValid() const;
    uint32_t findMemoryType(uint32_t bits, VkMemoryPropertyFlags props) const;

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
    std::vector<VkImage> images_;
    std::vector<VkImageView> views_;
    std::vector<VkFramebuffer> framebuffers_;
    std::vector<VkSemaphore> renderFinished_;  // one per swapchain image

    VkRenderPass renderPass_{};
    VkPipelineLayout pipelineLayout_{};
    VkPipeline pipeline_{};
    VkCommandPool pool_{};
    Frame frames_[FRAMES_IN_FLIGHT]{};
    uint32_t frameIndex_ = 0;
};
