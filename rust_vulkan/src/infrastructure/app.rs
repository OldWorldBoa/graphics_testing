#![allow(
    dead_code,
    unused_variables,
    clippy::manual_slice_size_calculation,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
    unsafe_op_in_unsafe_fn
)]

use anyhow::{anyhow, Result};
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::window as vk_window;
use winit::window::Window;

use crate::infrastructure::buffer::{create_index_buffer, create_vertex_buffer};
use crate::infrastructure::commands::{create_command_buffers, create_command_pool};
use crate::infrastructure::constants::{MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED};
use crate::infrastructure::framebuffer::create_framebuffers;
use crate::infrastructure::instance::create_instance;
use crate::infrastructure::logical_device::create_logical_device;
use crate::infrastructure::physical_device::pick_physical_device;
use crate::infrastructure::pipeline::{create_pipeline, create_render_pass};
use crate::infrastructure::swapchain::{create_swapchain, SwapchainInfo};

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
}

impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        data.physical_device = pick_physical_device(&instance, data.surface)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        data.swapchain_info = create_swapchain(
            window,
            &instance,
            &device,
            data.surface,
            data.physical_device,
        )?;
        data.render_pass =
            create_render_pass(&instance, &device, data.swapchain_info.swapchain_format)?;

        let (pipeline_layout, pipeline) = create_pipeline(
            &device,
            data.render_pass,
            data.swapchain_info.swapchain_extent,
        )?;
        data.pipeline_layout = pipeline_layout;
        data.pipeline = pipeline;
        data.framebuffers = create_framebuffers(
            &device,
            data.render_pass,
            &data.swapchain_info.swapchain_image_views,
            data.swapchain_info.swapchain_extent.height,
            data.swapchain_info.swapchain_extent.width,
        )?;
        data.command_pool =
            create_command_pool(&instance, &device, data.surface, data.physical_device)?;

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
            &instance,
            &device,
            data.command_pool,
            data.graphics_queue,
            data.physical_device,
        )?;
        data.vertex_buffer = vertex_buffer;
        data.vertex_buffer_memory = vertex_buffer_memory;

        let (index_buffer, index_buffer_memory) = create_index_buffer(
            &instance,
            &device,
            data.command_pool,
            data.graphics_queue,
            data.physical_device,
        )?;
        data.index_buffer = index_buffer;
        data.index_buffer_memory = index_buffer_memory;

        data.command_buffers = create_command_buffers(
            &device,
            data.command_pool,
            &data.framebuffers,
            data.render_pass,
            data.pipeline,
            data.vertex_buffer,
            data.index_buffer,
            data.swapchain_info.swapchain_extent,
        )?;
        create_sync_objects(&device, &mut data)?;

        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
        })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];

        self.device
            .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;

        let result = self.device.acquire_next_image_khr(
            self.data.swapchain_info.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .wait_for_fences(&[image_in_flight], true, u64::MAX)?;
        }

        self.data.images_in_flight[image_index] = in_flight_fence;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[in_flight_fence])?;

        self.device
            .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?;

        let swapchains = &[self.data.swapchain_info.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .queue_present_khr(self.data.present_queue, &present_info);
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
        self.data.swapchain_info = create_swapchain(
            window,
            &self.instance,
            &self.device,
            self.data.surface,
            self.data.physical_device,
        )?;
        self.data.render_pass = create_render_pass(
            &self.instance,
            &self.device,
            self.data.swapchain_info.swapchain_format,
        )?;
        let (pipeline_layout, pipeline) = create_pipeline(
            &self.device,
            self.data.render_pass,
            self.data.swapchain_info.swapchain_extent,
        )?;
        self.data.pipeline = pipeline;
        self.data.pipeline_layout = pipeline_layout;

        self.data.framebuffers = create_framebuffers(
            &self.device,
            self.data.render_pass,
            &self.data.swapchain_info.swapchain_image_views,
            self.data.swapchain_info.swapchain_extent.height,
            self.data.swapchain_info.swapchain_extent.width,
        )?;
        self.data.command_buffers = create_command_buffers(
            &self.device,
            self.data.command_pool,
            &self.data.framebuffers,
            self.data.render_pass,
            self.data.pipeline,
            self.data.vertex_buffer,
            self.data.index_buffer,
            self.data.swapchain_info.swapchain_extent,
        )?;
        self.data.images_in_flight.resize(
            self.data.swapchain_info.swapchain_images.len(),
            vk::Fence::null(),
        );

        Ok(())
    }

    /// Destroys our Vulkan app.
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

        self.data.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device.free_memory(self.data.index_buffer_memory, None);
        self.device.destroy_buffer(self.data.index_buffer, None);
        self.device.free_memory(self.data.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device.destroy_command_pool(self.data.command_pool, None);
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);

        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    /// Destroys the parts of our Vulkan app related to the swapchain.
    #[rustfmt::skip]
    unsafe fn destroy_swapchain(&mut self) {
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.data.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data.swapchain_info.swapchain_image_views.iter().for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain_info.swapchain, None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
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
    // Command Buffers
    pub command_buffers: Vec<vk::CommandBuffer>,
    // Sync Objects
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
}

unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
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
