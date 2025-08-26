use anyhow::Result;
use vk::DescriptorPool;
use vulkanalia::{prelude::v1_0::*, vk::DescriptorSet};

use crate::world::uniform::UniformBufferObject;

pub unsafe fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let bindings = &[ubo_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    let descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(descriptor_set_layout)
}

pub unsafe fn create_descriptor_pool(
    device: &Device,
    swapchain_image_len: u32,
) -> Result<DescriptorPool> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(swapchain_image_len);

    let pool_size = &[ubo_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_size)
        .max_sets(swapchain_image_len);

    Ok(device.create_descriptor_pool(&info, None)?)
}

pub unsafe fn create_descriptor_sets(
    device: &Device,
    uniform_buffers: &[vk::Buffer],
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

        device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok(desc_sets)
}
