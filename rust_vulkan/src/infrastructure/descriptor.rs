use anyhow::Result;
use vk::DescriptorPool;
use vulkanalia::{prelude::v1_0::*, vk::DescriptorSet};

use crate::{infrastructure::image::ImageBundle, world::uniform::UniformBufferObject};

/// Creates the descriptor set layout
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    Ok(device.create_descriptor_set_layout(&info, None)?)
}

/// Creates the descriptor pool
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_descriptor_pool(
    device: &Device,
    swapchain_image_len: u32,
) -> Result<DescriptorPool> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(swapchain_image_len);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(swapchain_image_len);

    let pool_size = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_size)
        .max_sets(swapchain_image_len);

    Ok(device.create_descriptor_pool(&info, None)?)
}

/// Creates the descriptor sets
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_descriptor_sets(
    device: &Device,
    texture_sampler: vk::Sampler,
    uniform_buffers: &[vk::Buffer],
    texture_bundle: &ImageBundle,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    swapchain_image_len: usize,
) -> Result<Vec<DescriptorSet>> {
    let layouts = vec![layout; swapchain_image_len];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);

    let desc_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..swapchain_image_len {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i])
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
            .image_view(texture_bundle.image_view)
            .sampler(texture_sampler);

        let image_infos = &[image_info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(desc_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos);

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok(desc_sets)
}
