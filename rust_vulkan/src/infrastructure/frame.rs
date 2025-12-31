use anyhow::Result;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::vk::DescriptorPool;
use vulkanalia::vk::DescriptorSetLayout;
use vulkanalia::vk::PhysicalDevice;
use vulkanalia::Device;

use crate::infrastructure::buffer::copy_buffer;
use crate::infrastructure::buffer::create_buffer;
use crate::infrastructure::buffer::create_index_buffer;
use crate::infrastructure::buffer::create_vertex_buffer;
use crate::infrastructure::commands::create_command_pool;
use crate::infrastructure::swapchain::SwapchainInfo;
use crate::world::camera::UniformBufferObject;
use crate::world::scene::SceneData;
use crate::world::vertex::Vertex;

//================================================
// Frame Bundle
//================================================
#[derive(Debug, Clone)]
pub struct FrameBundle {
    pub frame_buffer: vk::Framebuffer,
    pub command_pool: vk::CommandPool,
    pub primary_command_buffer: vk::CommandBuffer,
    pub entity_command_bundles: Vec<EntityCommandBundle>,
    pub uniform_buffer: vk::Buffer,
    pub uniform_buffer_memory: vk::DeviceMemory,
    pub descriptor_set: vk::DescriptorSet,
}

#[derive(Debug, Clone)]
pub struct EntityCommandBundle {
    pub secondary_command_buffer: vk::CommandBuffer,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
}

impl FrameBundle {
    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn update_uniform_buffer(
        &self,
        device: &Device,
        scene_data: &SceneData,
    ) -> Result<()> {
        let memory = device.map_memory(
            self.uniform_buffer_memory,
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&scene_data.camera.get_ubo(), memory.cast(), 1);

        device.unmap_memory(self.uniform_buffer_memory);

        Ok(())
    }

    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn update_vertex_buffers(
        &self,
        instance: &Instance,
        device: &Device,
        physical_device: PhysicalDevice,
        graphics_queue: vk::Queue,
        scene_data: &SceneData,
    ) -> Result<()> {
        for i in 0..scene_data.entities.len() {
            let vertices = &scene_data.entities[i].vertex_data;
            let size = (size_of::<Vertex>() * vertices.len()) as u64;
            // Create staging buffers
            let (staging_buffer, staging_buffer_memory) = create_buffer(
                instance,
                device,
                physical_device,
                size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            )?;

            // Copy (staging)
            let memory =
                device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

            memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
            device.unmap_memory(staging_buffer_memory);

            // Copy (vertex)
            copy_buffer(
                device,
                self.command_pool,
                graphics_queue,
                staging_buffer,
                self.entity_command_bundles[i].vertex_buffer,
                size,
            )?;

            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }
        Ok(())
    }
}

/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_framebundles(
    instance: &Instance,
    device: &Device,
    graphics_queue: vk::Queue,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    render_pass: vk::RenderPass,
    swapchain_info: &SwapchainInfo,
    depth_view: vk::ImageView,
    texture_view: vk::ImageView,
    sampling_view: vk::ImageView,
    pool: DescriptorPool,
    layout: DescriptorSetLayout,
    sampler: vk::Sampler,
    scene_data: &SceneData,
) -> Result<Vec<FrameBundle>> {
    let mut framebundles = vec![];
    let len = swapchain_info.swapchain_image_views.len();
    let layouts = vec![layout; len];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);

    let desc_sets = device.allocate_descriptor_sets(&info)?;

    for (i, swapchain_view) in swapchain_info.swapchain_image_views.iter().enumerate() {
        let command_pool = create_command_pool(instance, device, surface, physical_device)?;

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let primary_command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

        let mut entity_command_bundles = vec![];
        for j in 0..scene_data.entities.len() {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

            let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
                instance,
                device,
                graphics_queue,
                physical_device,
                command_pool,
                &scene_data.entities[j].vertex_data,
            )?;

            let (index_buffer, index_buffer_memory) = create_index_buffer(
                instance,
                device,
                command_pool,
                graphics_queue,
                physical_device,
                &scene_data.entities[j].vertex_indices,
            )?;

            entity_command_bundles.push(EntityCommandBundle {
                secondary_command_buffer: command_buffer,
                vertex_buffer,
                vertex_buffer_memory,
                index_buffer,
                index_buffer_memory,
            });
        }

        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffer)
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(desc_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_view)
            .sampler(sampler);

        let image_infos = &[image_info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(desc_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos);

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);

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
            primary_command_buffer,
            entity_command_bundles,
            uniform_buffer,
            uniform_buffer_memory,
            descriptor_set: desc_sets[i],
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
