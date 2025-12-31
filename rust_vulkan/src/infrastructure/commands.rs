use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{PhysicalDevice, SurfaceKHR};

use crate::infrastructure::frame::FrameBundle;
use crate::infrastructure::queue_family_indices::QueueFamilyIndices;
use crate::world::entity::entity::Entity;
use crate::world::scene::SceneData;
use crate::world::vertex::Mat4;

//================================================
// Command Pool
//================================================

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
pub unsafe fn update_command_buffer(
    device: &Device,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    extent: vk::Extent2D,
    framebundle: &FrameBundle,
    scene_data: &SceneData,
) -> Result<()> {
    device.reset_command_pool(framebundle.command_pool, vk::CommandPoolResetFlags::empty())?;
    let command_buffer = framebundle.primary_command_buffer;

    // Commands
    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

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

    device.cmd_begin_render_pass(
        command_buffer,
        &info,
        vk::SubpassContents::SECONDARY_COMMAND_BUFFERS,
    );

    for (i, entity) in scene_data.entities.iter().enumerate() {
        update_secondary_command_buffer(
            device,
            render_pass,
            pipeline,
            pipeline_layout,
            framebundle,
            i,
            entity,
        )?;
    }

    let mut secondary_command_buffers = vec![];
    for bundle in framebundle.entity_command_bundles.iter() {
        secondary_command_buffers.push(bundle.secondary_command_buffer);
    }

    device.cmd_execute_commands(command_buffer, &secondary_command_buffers);
    device.cmd_end_render_pass(command_buffer);
    device.end_command_buffer(command_buffer)?;

    Ok(())
}

unsafe fn update_secondary_command_buffer(
    device: &Device,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    framebundle: &FrameBundle,
    model_index: usize,
    entity: &Entity,
) -> Result<()> {
    let entity_command_bundle = &framebundle.entity_command_bundles[model_index];
    let command_buffer = entity_command_bundle.secondary_command_buffer;

    let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
        .render_pass(render_pass)
        .subpass(0)
        .framebuffer(framebundle.frame_buffer);

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
        .inheritance_info(&inheritance_info);

    device.begin_command_buffer(command_buffer, &info)?;
    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
    device.cmd_bind_vertex_buffers(
        command_buffer,
        0,
        &[entity_command_bundle.vertex_buffer],
        &[0],
    );
    device.cmd_bind_index_buffer(
        command_buffer,
        entity_command_bundle.index_buffer,
        0,
        vk::IndexType::UINT32,
    );
    device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        pipeline_layout,
        0,
        &[framebundle.descriptor_set],
        &[],
    );

    let model_bytes = std::slice::from_raw_parts(
        &entity.transform as *const Mat4 as *const u8,
        size_of::<Mat4>(),
    );

    device.cmd_push_constants(
        command_buffer,
        pipeline_layout,
        vk::ShaderStageFlags::VERTEX,
        0,
        model_bytes,
    );
    device.cmd_push_constants(
        command_buffer,
        pipeline_layout,
        vk::ShaderStageFlags::FRAGMENT,
        64,
        &entity.opacity.to_ne_bytes()[..],
    );
    device.cmd_draw_indexed(
        command_buffer,
        entity.vertex_indices.len() as u32,
        1,
        0,
        0,
        0,
    );

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
