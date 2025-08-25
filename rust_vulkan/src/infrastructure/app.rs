#![allow(
    dead_code,
    unused_variables,
    clippy::manual_slice_size_calculation,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
    unsafe_op_in_unsafe_fn
)]

use anyhow::{anyhow, Result};
use cgmath::{point3, vec3, Deg};
use std::time::Instant;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::window as vk_window;
use winit::window::Window;

use crate::infrastructure::buffer::{
    create_index_buffer, create_uniform_buffers, create_vertex_buffer,
};
use crate::infrastructure::commands::{create_command_buffers, create_command_pool};
use crate::infrastructure::constants::{MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED};
use crate::infrastructure::descriptor_set::create_descriptor_set_layout;
use crate::infrastructure::framebuffer::create_framebuffers;
use crate::infrastructure::instance::create_instance;
use crate::infrastructure::logical_device::create_logical_device;
use crate::infrastructure::physical_device::pick_physical_device;
use crate::infrastructure::pipeline::{create_pipeline, create_render_pass};
use crate::infrastructure::swapchain::{create_swapchain, SwapchainInfo};
use crate::world::uniform::UniformBufferObject;
use crate::world::vertex::{Mat4, Vertex};

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub infrastructure: AppInfrastructure,
    pub data: AppData,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    // animation
    pub start: Instant,
}

impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut infrastructure = AppInfrastructure::default();
        let instance = create_instance(window, &entry, &mut infrastructure)?;
        infrastructure.surface = vk_window::create_surface(&instance, &window, &window)?;
        infrastructure.physical_device = pick_physical_device(&instance, infrastructure.surface)?;
        let device = create_logical_device(&entry, &instance, &mut infrastructure)?;
        infrastructure.swapchain_info = create_swapchain(
            window,
            &instance,
            &device,
            infrastructure.surface,
            infrastructure.physical_device,
        )?;
        infrastructure.render_pass = create_render_pass(
            &instance,
            &device,
            infrastructure.swapchain_info.swapchain_format,
        )?;
        infrastructure.descriptor_set_layout = create_descriptor_set_layout(&device)?;

        let (pipeline_layout, pipeline) = create_pipeline(
            &device,
            infrastructure.render_pass,
            infrastructure.descriptor_set_layout,
            infrastructure.swapchain_info.swapchain_extent,
        )?;
        infrastructure.pipeline_layout = pipeline_layout;
        infrastructure.pipeline = pipeline;
        infrastructure.framebuffers = create_framebuffers(
            &device,
            infrastructure.render_pass,
            &infrastructure.swapchain_info.swapchain_image_views,
            infrastructure.swapchain_info.swapchain_extent.height,
            infrastructure.swapchain_info.swapchain_extent.width,
        )?;
        infrastructure.command_pool = create_command_pool(
            &instance,
            &device,
            infrastructure.surface,
            infrastructure.physical_device,
        )?;

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
            &instance,
            &device,
            infrastructure.command_pool,
            infrastructure.graphics_queue,
            infrastructure.physical_device,
        )?;
        infrastructure.vertex_buffer = vertex_buffer;
        infrastructure.vertex_buffer_memory = vertex_buffer_memory;

        let (index_buffer, index_buffer_memory) = create_index_buffer(
            &instance,
            &device,
            infrastructure.command_pool,
            infrastructure.graphics_queue,
            infrastructure.physical_device,
        )?;
        infrastructure.index_buffer = index_buffer;
        infrastructure.index_buffer_memory = index_buffer_memory;

        let uniform_infrastructure = create_uniform_buffers(
            &instance,
            &device,
            infrastructure.physical_device,
            infrastructure.swapchain_info.swapchain_images.len(),
        )?;

        infrastructure.uniform_buffers.clear();
        infrastructure.uniform_buffers_memory.clear();
        for (uniform_buffer, uniform_buffer_memory) in uniform_infrastructure {
            infrastructure.uniform_buffers.push(uniform_buffer);
            infrastructure
                .uniform_buffers_memory
                .push(uniform_buffer_memory);
        }

        infrastructure.command_buffers = create_command_buffers(
            &device,
            infrastructure.command_pool,
            &infrastructure.framebuffers,
            infrastructure.render_pass,
            infrastructure.pipeline,
            infrastructure.vertex_buffer,
            infrastructure.index_buffer,
            infrastructure.swapchain_info.swapchain_extent,
        )?;
        create_sync_objects(&device, &mut infrastructure)?;

        Ok(Self {
            entry,
            instance,
            infrastructure,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        let in_flight_fence = self.infrastructure.in_flight_fences[self.frame];

        self.device
            .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;

        let result = self.device.acquire_next_image_khr(
            self.infrastructure.swapchain_info.swapchain,
            u64::MAX,
            self.infrastructure.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        let image_in_flight = self.infrastructure.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .wait_for_fences(&[image_in_flight], true, u64::MAX)?;
        }

        self.infrastructure.images_in_flight[image_index] = in_flight_fence;

        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.infrastructure.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.infrastructure.command_buffers[image_index]];
        let signal_semaphores = &[self.infrastructure.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[in_flight_fence])?;

        self.device.queue_submit(
            self.infrastructure.graphics_queue,
            &[submit_info],
            in_flight_fence,
        )?;

        let swapchains = &[self.infrastructure.swapchain_info.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .queue_present_khr(self.infrastructure.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    /// Recreates the swapchain for our Vulkan app.
    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        self.infrastructure.swapchain_info = create_swapchain(
            window,
            &self.instance,
            &self.device,
            self.infrastructure.surface,
            self.infrastructure.physical_device,
        )?;
        self.infrastructure.render_pass = create_render_pass(
            &self.instance,
            &self.device,
            self.infrastructure.swapchain_info.swapchain_format,
        )?;
        self.infrastructure.descriptor_set_layout = create_descriptor_set_layout(&self.device)?;
        let (pipeline_layout, pipeline) = create_pipeline(
            &self.device,
            self.infrastructure.render_pass,
            self.infrastructure.descriptor_set_layout,
            self.infrastructure.swapchain_info.swapchain_extent,
        )?;
        self.infrastructure.pipeline = pipeline;
        self.infrastructure.pipeline_layout = pipeline_layout;

        self.infrastructure.framebuffers = create_framebuffers(
            &self.device,
            self.infrastructure.render_pass,
            &self.infrastructure.swapchain_info.swapchain_image_views,
            self.infrastructure.swapchain_info.swapchain_extent.height,
            self.infrastructure.swapchain_info.swapchain_extent.width,
        )?;

        let uniform_infrastructure = create_uniform_buffers(
            &self.instance,
            &self.device,
            self.infrastructure.physical_device,
            self.infrastructure.swapchain_info.swapchain_images.len(),
        )?;

        self.infrastructure.uniform_buffers.clear();
        self.infrastructure.uniform_buffers_memory.clear();
        for (uniform_buffer, uniform_buffer_memory) in uniform_infrastructure {
            self.infrastructure.uniform_buffers.push(uniform_buffer);
            self.infrastructure
                .uniform_buffers_memory
                .push(uniform_buffer_memory);
        }

        self.infrastructure.command_buffers = create_command_buffers(
            &self.device,
            self.infrastructure.command_pool,
            &self.infrastructure.framebuffers,
            self.infrastructure.render_pass,
            self.infrastructure.pipeline,
            self.infrastructure.vertex_buffer,
            self.infrastructure.index_buffer,
            self.infrastructure.swapchain_info.swapchain_extent,
        )?;
        self.infrastructure.images_in_flight.resize(
            self.infrastructure.swapchain_info.swapchain_images.len(),
            vk::Fence::null(),
        );

        Ok(())
    }

    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        let time = self.start.elapsed().as_secs_f32();

        let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0) * time);

        let view = Mat4::look_at_rh(
            point3(2.0, 2.0, 2.0),
            point3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        );

        let mut proj = cgmath::perspective(
            Deg(45.0),
            self.infrastructure.swapchain_info.swapchain_extent.width as f32
                / self.infrastructure.swapchain_info.swapchain_extent.height as f32,
            0.1,
            10.0,
        );

        proj[1][1] *= -1.0;

        let ubo = UniformBufferObject { model, view, proj };

        let memory = self.device.map_memory(
            self.infrastructure.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        );

        memcpy(&ubo, memory.cast(), 1);

        Ok(())
    }

    /// Destroys our Vulkan app.
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();
        self.device.destroy_descriptor_set_layout(self.infrastructure.descriptor_set_layout, None);
        self.infrastructure.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
        self.infrastructure.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.infrastructure.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device.free_memory(self.infrastructure.index_buffer_memory, None);
        self.device.destroy_buffer(self.infrastructure.index_buffer, None);
        self.device.free_memory(self.infrastructure.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.infrastructure.vertex_buffer, None);
        self.infrastructure.uniform_buffers.iter().for_each(|b| self.device.destroy_buffer(*b, None));
        self.infrastructure.uniform_buffers_memory.iter().for_each(|m| self.device.free_memory(*m, None));
        self.device.destroy_command_pool(self.infrastructure.command_pool, None);
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.infrastructure.surface, None);

        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.infrastructure.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    /// Destroys the parts of our Vulkan app related to the swapchain.
    #[rustfmt::skip]
    unsafe fn destroy_swapchain(&mut self) {
        self.device.free_command_buffers(self.infrastructure.command_pool, &self.infrastructure.command_buffers);
        self.infrastructure.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.infrastructure.pipeline, None);
        self.device.destroy_pipeline_layout(self.infrastructure.pipeline_layout, None);
        self.device.destroy_render_pass(self.infrastructure.render_pass, None);
        self.infrastructure.swapchain_info.swapchain_image_views.iter().for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.infrastructure.swapchain_info.swapchain, None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppInfrastructure {
    // Debug
    pub messenger: vk::DebugUtilsMessengerEXT,
    // Surface
    pub surface: vk::SurfaceKHR,
    // Physical Device / Logical Device
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    // Swapchain
    pub swapchain_info: SwapchainInfo,
    // Pipeline
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    // Framebuffers
    pub framebuffers: Vec<vk::Framebuffer>,
    // Command Pool
    pub command_pool: vk::CommandPool,
    // Buffers
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    // Command Buffers
    pub command_buffers: Vec<vk::CommandBuffer>,
    // Sync Objects
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    // Descriptor Sets
    pub descriptor_set_layout: vk::DescriptorSetLayout,
}

#[derive(Debug, Clone)]
pub struct AppData {
    uniform_data: UniformBufferObject,
    vertex_data: [Vertex; 4],
    vertex_indices: [u16; 6],
}

unsafe fn create_sync_objects(device: &Device, data: &mut AppInfrastructure) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.image_available_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        data.render_finished_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);

        data.in_flight_fences
            .push(device.create_fence(&fence_info, None)?);
    }

    data.images_in_flight = data
        .swapchain_info
        .swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}
