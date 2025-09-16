use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::Device;

use crate::infrastructure::commands::create_command_buffer;
use crate::infrastructure::commands::create_command_pool;
use crate::infrastructure::swapchain::SwapchainInfo;

//================================================
// Frame Bundle
//================================================
#[derive(Debug, Clone)]
pub struct FrameBundle {
    pub frame_buffer: vk::Framebuffer,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub secondary_command_buffers: Vec<vk::CommandBuffer>,
}

/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_framebundles(
    instance: &Instance,
    device: &Device,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    render_pass: vk::RenderPass,
    swapchain_info: &SwapchainInfo,
    depth_view: vk::ImageView,
    sampling_view: vk::ImageView,
    num_models: u32,
) -> Result<Vec<FrameBundle>> {
    let mut framebundles = vec![];

    for swapchain_view in swapchain_info.swapchain_image_views.iter() {
        let command_pool = create_command_pool(instance, device, surface, physical_device)?;
        let mut secondary_command_buffers = vec![];

        for i in 0..num_models {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

            secondary_command_buffers.push(command_buffer);
        }

        framebundles.push(FrameBundle {
            frame_buffer: create_framebuffer(
                device,
                render_pass,
                swapchain_view,
                depth_view,
                sampling_view,
                swapchain_info.swapchain_extent.height,
                swapchain_info.swapchain_extent.width,
            )?,
            command_pool,
            command_buffers: vec![create_command_buffer(device, command_pool)?],
            secondary_command_buffers,
        });
    }

    Ok(framebundles)
}

//================================================
// Framebuffers
//================================================

/// Creates a framebuffer
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_framebuffer(
    device: &Device,
    render_pass: vk::RenderPass,
    swapchain_view: &vk::ImageView,
    depth_view: vk::ImageView,
    sampling_view: vk::ImageView,
    height: u32,
    width: u32,
) -> Result<vk::Framebuffer> {
    let attachments = &[sampling_view, depth_view, *swapchain_view];
    let create_info = vk::FramebufferCreateInfo::builder()
        .render_pass(render_pass)
        .attachments(attachments)
        .width(width)
        .height(height)
        .layers(1);

    Ok(device.create_framebuffer(&create_info, None)?)
}
