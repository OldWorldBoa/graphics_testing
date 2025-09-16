use anyhow::{anyhow, Result};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{CommandPool, PhysicalDevice};

use crate::infrastructure::commands::{begin_single_time_commands, end_single_time_commands};
use crate::world::uniform::UniformBufferObject;
use crate::world::vertex::Vertex;

//================================================
// Buffers
//================================================

/// Creates buffers for vertices
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    command_pool: CommandPool,
    graphics_queue: vk::Queue,
    physical_device: PhysicalDevice,
    vertices: &[Vertex],
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
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
    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_memory);

    // Create (vertex)
    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // Copy (vertex)
    copy_buffer(
        device,
        command_pool,
        graphics_queue,
        staging_buffer,
        vertex_buffer,
        size,
    )?;

    // Cleanup
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((vertex_buffer, vertex_buffer_memory))
}

/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn update_vertex_buffer(
    vertices: &[Vertex],
    vertex_buffer: vk::Buffer,
    instance: &Instance,
    device: &Device,
    physical_device: PhysicalDevice,
    command_pool: CommandPool,
    graphics_queue: vk::Queue,
) -> Result<()> {
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
    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
    device.unmap_memory(staging_buffer_memory);

    // Copy (vertex)
    copy_buffer(
        device,
        command_pool,
        graphics_queue,
        staging_buffer,
        vertex_buffer,
        size,
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

/// Creates buffers for indices
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    command_pool: CommandPool,
    graphics_queue: vk::Queue,
    physical_device: PhysicalDevice,
    indices: &[u32],
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    // Create (staging)

    let size = (size_of::<u32>() * indices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    // Copy (staging)

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(indices.as_ptr(), memory.cast(), indices.len());

    device.unmap_memory(staging_buffer_memory);

    // Create (index)

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // Copy (index)
    copy_buffer(
        device,
        command_pool,
        graphics_queue,
        staging_buffer,
        index_buffer,
        size,
    )?;

    // Cleanup

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((index_buffer, index_buffer_memory))
}

/// Creates buffers for uniforms
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    physical_device: PhysicalDevice,
    num_buffers: usize,
) -> Result<Vec<(vk::Buffer, vk::DeviceMemory)>> {
    let mut data = vec![];

    for _ in 0..num_buffers {
        data.push(create_buffer(
            instance,
            device,
            physical_device,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?);
    }

    Ok(data)
}

//================================================
// Shared (Buffers)
//================================================

/// Creates generic buffers
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    // Buffer

    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    // Memory

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            physical_device,
            properties,
            requirements,
        )?);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

unsafe fn copy_buffer(
    device: &Device,
    command_pool: CommandPool,
    graphics_queue: vk::Queue,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

    end_single_time_commands(device, graphics_queue, command_pool, command_buffer)?;

    Ok(())
}

/// Gets the memory type index
///
/// # Safety
/// Check the vulkan docs for safety info
pub unsafe fn get_memory_type_index(
    instance: &Instance,
    physical_device: PhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory = instance.get_physical_device_memory_properties(physical_device);
    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}
