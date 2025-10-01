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
use winit::dpi::PhysicalPosition;
use winit::window::Window;

use crate::infrastructure::commands::{create_command_pool, update_command_buffer};
use crate::infrastructure::constants::{MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED};
use crate::infrastructure::descriptor::{create_descriptor_pool, create_descriptor_set_layout};
use crate::infrastructure::frame::{create_framebundles, FrameBundle};
use crate::infrastructure::image::{
    create_depth_image, create_sampling_image, create_texture_image, create_texture_sampler,
    ImageBundle,
};
use crate::infrastructure::instance::create_instance;
use crate::infrastructure::logical_device::create_logical_device;
use crate::infrastructure::physical_device::{get_max_msaa_samples, pick_physical_device};
use crate::infrastructure::pipeline::{create_pipeline, create_render_pass};
use crate::infrastructure::swapchain::{create_swapchain, SwapchainInfo};
use crate::world::scene::Scene;

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub infrastructure: AppInfrastructure,
    pub scene: Scene,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
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
    pub msaa_samples: vk::SampleCountFlags,

    // Swapchain
    pub swapchain_info: SwapchainInfo,

    // Pipeline
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub static_command_pool: vk::CommandPool,

    // Frame Bundles (Buffer, Command Pool, Command Buffer)
    pub framebundles: Vec<FrameBundle>,

    // Image Bundles
    pub color_sample_bundle: ImageBundle,
    pub depth_bundle: ImageBundle,
    pub texture_bundle: ImageBundle,

    // Sync Objects
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    // Descriptors
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    // Sampler
    pub texture_sampler: vk::Sampler,
}

impl App {
    /// Creates our Vulkan app.
    ///
    /// # Safety
    /// Check the vulkan docs for safety information
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut infrastructure = AppInfrastructure::default();
        let instance = create_instance(window, &entry, &mut infrastructure)?;
        infrastructure.surface = vk_window::create_surface(&instance, &window, &window)?;
        infrastructure.physical_device = pick_physical_device(&instance, infrastructure.surface)?;
        infrastructure.msaa_samples =
            get_max_msaa_samples(&instance, infrastructure.physical_device);

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
            infrastructure.physical_device,
            infrastructure.swapchain_info.swapchain_format,
            infrastructure.msaa_samples,
        )?;

        infrastructure.descriptor_set_layout = create_descriptor_set_layout(&device)?;

        let (pipeline_layout, pipeline) = create_pipeline(
            &device,
            infrastructure.render_pass,
            infrastructure.descriptor_set_layout,
            infrastructure.swapchain_info.swapchain_extent,
            infrastructure.msaa_samples,
        )?;
        infrastructure.pipeline_layout = pipeline_layout;
        infrastructure.pipeline = pipeline;

        infrastructure.static_command_pool = create_command_pool(
            &instance,
            &device,
            infrastructure.surface,
            infrastructure.physical_device,
        )?;

        let scene = Scene::create_scene(
            infrastructure.swapchain_info.swapchain_extent.width as f32
                / infrastructure.swapchain_info.swapchain_extent.height as f32,
        )?;

        infrastructure.color_sample_bundle = create_sampling_image(
            &instance,
            &device,
            infrastructure.physical_device,
            &infrastructure.swapchain_info,
            infrastructure.msaa_samples,
        )?;

        infrastructure.depth_bundle = create_depth_image(
            &device,
            &instance,
            infrastructure.physical_device,
            infrastructure.graphics_queue,
            infrastructure.swapchain_info.swapchain_extent,
            infrastructure.msaa_samples,
        )?;

        infrastructure.texture_bundle = create_texture_image(
            &scene.scene_data.entities[0].texture,
            &device,
            &instance,
            infrastructure.physical_device,
            infrastructure.static_command_pool,
            infrastructure.graphics_queue,
        )?;

        infrastructure.texture_sampler =
            create_texture_sampler(&device, infrastructure.texture_bundle.mip_levels)?;

        infrastructure.descriptor_pool = create_descriptor_pool(
            &device,
            infrastructure.swapchain_info.swapchain_images.len() as u32,
        )?;

        infrastructure.framebundles = create_framebundles(
            &instance,
            &device,
            infrastructure.graphics_queue,
            infrastructure.surface,
            infrastructure.physical_device,
            infrastructure.render_pass,
            &infrastructure.swapchain_info,
            infrastructure.depth_bundle.image_view,
            infrastructure.texture_bundle.image_view,
            infrastructure.color_sample_bundle.image_view,
            infrastructure.descriptor_pool,
            infrastructure.descriptor_set_layout,
            infrastructure.texture_sampler,
            &scene.scene_data,
        )?;

        create_sync_objects(&device, &mut infrastructure)?;

        Ok(Self {
            entry,
            instance,
            infrastructure,
            device,
            frame: 0,
            resized: false,
            scene,
        })
    }

    /// Renders a frame for our Vulkan app.
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.scene.spinner.work(&mut self.scene.scene_data);

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
        self.infrastructure.framebundles[image_index]
            .update_uniform_buffer(&self.device, &self.scene.scene_data)?;
        self.infrastructure.framebundles[image_index].update_vertex_buffer(
            &self.instance,
            &self.device,
            self.infrastructure.physical_device,
            self.infrastructure.graphics_queue,
            &self.scene.scene_data,
        )?;

        update_command_buffer(
            &self.device,
            self.infrastructure.pipeline_layout,
            self.infrastructure.render_pass,
            self.infrastructure.pipeline,
            self.infrastructure.swapchain_info.swapchain_extent,
            &self.infrastructure.framebundles[image_index],
            &self.scene.scene_data,
        )?;

        let wait_semaphores = &[self.infrastructure.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers =
            &[self.infrastructure.framebundles[image_index].primary_command_buffer];
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
            self.infrastructure.physical_device,
            self.infrastructure.swapchain_info.swapchain_format,
            self.infrastructure.msaa_samples,
        )?;

        let (pipeline_layout, pipeline) = create_pipeline(
            &self.device,
            self.infrastructure.render_pass,
            self.infrastructure.descriptor_set_layout,
            self.infrastructure.swapchain_info.swapchain_extent,
            self.infrastructure.msaa_samples,
        )?;
        self.infrastructure.pipeline = pipeline;
        self.infrastructure.pipeline_layout = pipeline_layout;

        self.infrastructure.color_sample_bundle = create_sampling_image(
            &self.instance,
            &self.device,
            self.infrastructure.physical_device,
            &self.infrastructure.swapchain_info,
            self.infrastructure.msaa_samples,
        )?;

        self.infrastructure.depth_bundle = create_depth_image(
            &self.device,
            &self.instance,
            self.infrastructure.physical_device,
            self.infrastructure.graphics_queue,
            self.infrastructure.swapchain_info.swapchain_extent,
            self.infrastructure.msaa_samples,
        )?;

        self.infrastructure.texture_bundle = create_texture_image(
            &self.scene.scene_data.entities[0].texture,
            &self.device,
            &self.instance,
            self.infrastructure.physical_device,
            self.infrastructure.static_command_pool,
            self.infrastructure.graphics_queue,
        )?;

        self.infrastructure.descriptor_pool = create_descriptor_pool(
            &self.device,
            self.infrastructure.swapchain_info.swapchain_images.len() as u32,
        )?;

        self.infrastructure.framebundles = create_framebundles(
            &self.instance,
            &self.device,
            self.infrastructure.graphics_queue,
            self.infrastructure.surface,
            self.infrastructure.physical_device,
            self.infrastructure.render_pass,
            &self.infrastructure.swapchain_info,
            self.infrastructure.depth_bundle.image_view,
            self.infrastructure.texture_bundle.image_view,
            self.infrastructure.color_sample_bundle.image_view,
            self.infrastructure.descriptor_pool,
            self.infrastructure.descriptor_set_layout,
            self.infrastructure.texture_sampler,
            &self.scene.scene_data,
        )?;

        self.infrastructure.images_in_flight.resize(
            self.infrastructure.swapchain_info.swapchain_images.len(),
            vk::Fence::null(),
        );

        Ok(())
    }

    /// Destroys our Vulkan app.
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

        self.device.destroy_sampler(self.infrastructure.texture_sampler, None);
        self.infrastructure.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
        self.infrastructure.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.infrastructure.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device.destroy_command_pool(self.infrastructure.static_command_pool, None);
        self.device.destroy_descriptor_set_layout(self.infrastructure.descriptor_set_layout, None);
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
        self.device.destroy_descriptor_pool(self.infrastructure.descriptor_pool, None);

        self.device.destroy_image(self.infrastructure.color_sample_bundle.image, None);
        self.device.destroy_image_view(self.infrastructure.color_sample_bundle.image_view, None);
        self.device.free_memory(self.infrastructure.color_sample_bundle.image_memory, None);

        self.device.destroy_image(self.infrastructure.texture_bundle.image, None);
        self.device.destroy_image_view(self.infrastructure.texture_bundle.image_view, None);
        self.device.free_memory(self.infrastructure.texture_bundle.image_memory, None);

        self.device.destroy_image(self.infrastructure.depth_bundle.image, None);
        self.device.destroy_image_view(self.infrastructure.depth_bundle.image_view, None);
        self.device.free_memory(self.infrastructure.depth_bundle.image_memory, None);

        self.infrastructure.framebundles.iter().for_each(|f| {
            self.device.free_command_buffers(f.command_pool, &[f.primary_command_buffer]);
            self.device.free_memory(f.uniform_buffer_memory, None);
            self.device.destroy_buffer(f.uniform_buffer, None);
            self.device.free_memory(f.index_buffer_memory, None);
            self.device.destroy_buffer(f.index_buffer, None);
            self.device.free_memory(f.vertex_buffer_memory, None);
            self.device.destroy_buffer(f.vertex_buffer, None);
            self.device.destroy_command_pool(f.command_pool, None);
            self.device.destroy_framebuffer(f.frame_buffer, None);
        });

        self.device.destroy_pipeline(self.infrastructure.pipeline, None);
        self.device.destroy_pipeline_layout(self.infrastructure.pipeline_layout, None);
        self.device.destroy_render_pass(self.infrastructure.render_pass, None);
        self.infrastructure.swapchain_info.swapchain_image_views.iter().for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.infrastructure.swapchain_info.swapchain, None);
    }

    pub fn handle_keyboard(&self, keyCode: winit::keyboard::KeyCode) {}

    pub fn handle_mouse(&self, mousePosition: PhysicalPosition<f64>) {
        let height = self.infrastructure.swapchain_info.swapchain_extent.height as f32;
        let width = self.infrastructure.swapchain_info.swapchain_extent.width as f32;

        let delta_x = (100.0 - mousePosition.x) as f32;
        let delta_y = (100.0 - mousePosition.y) as f32;
        if (delta_x.abs() > 0.005) {
            let x_angle = self.scene.scene_data.camera.fov * (delta_x / width);
            println!("Mouse change x: {delta_x} = {:?}", x_angle);
        }
        if (delta_y.abs() > 0.005) {
            let y_angle = self.scene.scene_data.camera.fov * (delta_y / height);
            println!("Mouse change y: {delta_y} = {:?}", y_angle);
        }
    }
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
