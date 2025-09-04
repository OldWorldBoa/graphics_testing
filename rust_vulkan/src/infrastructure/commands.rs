use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{PhysicalDevice, SurfaceKHR};

use crate::infrastructure::frame::FrameBundle;
use crate::infrastructure::queue_family_indices::QueueFamilyIndices;

//================================================
// Command Pool
//================================================

pub unsafe fn create_command_pools(
    instance: &Instance,
    device: &Device,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
    swapchain_img_len: usize,
) -> Result<(vk::CommandPool, Vec<vk::CommandPool>)> {
    let main_cmd_pool = create_command_pool(instance, device, surface, physical_device)?;
    let mut swp_chain_cmd_pools = vec![];

    for _ in 0..swapchain_img_len {
        swp_chain_cmd_pools.push(create_command_pool(
            instance,
            device,
            surface,
            physical_device,
        )?);
    }

    Ok((main_cmd_pool, swp_chain_cmd_pools))
}

/// # Safety
/// Check the vulkan spec for safety info
pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, surface, physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&info, None)?)
}

//================================================
// Command Buffers
//================================================

/// # Safety
/// Check the vulkan spec for safety info
pub unsafe fn create_command_buffer(
    device: &Device,
    command_pool: vk::CommandPool,
) -> Result<vk::CommandBuffer> {
    // Allocate
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    Ok(device.allocate_command_buffers(&allocate_info)?[0])
}

/// # Safety
/// Check the vulkan spec for safety info
pub unsafe fn create_command_buffers(
    device: &Device,
    command_pool: vk::CommandPool,
    framebuffers: &[vk::Framebuffer],
    swapchain_img_len: usize,
) -> Result<Vec<vk::CommandBuffer>> {
    let command_buffers = vec![];
    for idx in 0..swapchain_img_len {
        // Allocate
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32);

        let command_buffer = device.allocate_command_buffers(&allocate_info)?;
    }

    Ok(command_buffers)
}

/// # Safety
/// Check the vulkan spec for safety info
pub unsafe fn update_command_buffer(
    device: &Device,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    descriptor_sets: vk::DescriptorSet,
    extent: vk::Extent2D,
    indices_len: u32,
    framebundle: &FrameBundle,
) -> Result<()> {
    device.reset_command_pool(framebundle.command_pool, vk::CommandPoolResetFlags::empty())?;
    framebundle.command_buffers.iter().for_each(|c| {
        device.reset_command_buffer(*c, vk::CommandBufferResetFlags::empty());
    });

    // Commands
    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    let command_buffer = framebundle.command_buffers[0];

    device.begin_command_buffer(command_buffer, &info)?;

    let render_area = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };

    let depth_clear_value = vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        },
    };

    let clear_values = &[color_clear_value, depth_clear_value];
    let info = vk::RenderPassBeginInfo::builder()
        .render_pass(render_pass)
        .framebuffer(framebundle.frame_buffer)
        .render_area(render_area)
        .clear_values(clear_values);

    device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
    device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer], &[0]);
    device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
    device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        pipeline_layout,
        0,
        &[descriptor_sets],
        &[],
    );
    device.cmd_draw_indexed(command_buffer, indices_len, 1, 0, 0, 0);
    device.cmd_end_render_pass(command_buffer);

    device.end_command_buffer(command_buffer)?;

    Ok(())
}

//================================================
// Command Helpers
//================================================

/// # Safety
/// Check the vulkan spec for safety info
pub unsafe fn begin_single_time_commands(
    device: &Device,
    command_pool: vk::CommandPool,
) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&info)?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    Ok(command_buffer)
}

/// # Safety
/// Check the vulkan spec for safety info
pub unsafe fn end_single_time_commands(
    device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    // SAFETY: https://registry.khronos.org/vulkan/specs/latest/man/html/vkEndCommandBuffer.html
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    // SAFETY: https://www.khronos.org/registry/vulkan/specs/latest/man/html/vkQueueSubmit.html
    device.queue_submit(graphics_queue, &[info], vk::Fence::null())?;
    // SAFETY: https://www.khronos.org/registry/vulkan/specs/latest/man/html/vkQueueWaitIdle.html
    device.queue_wait_idle(graphics_queue)?;

    // SAFETY: https://www.khronos.org/registry/vulkan/specs/latest/man/html/vkFreeCommandBuffers.html
    device.free_command_buffers(command_pool, command_buffers);

    Ok(())
}
