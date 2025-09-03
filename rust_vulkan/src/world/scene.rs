use anyhow::Result;
use cgmath::{point3, vec2, vec3, Deg};
use std::time::Instant;
use vk::PhysicalDevice;
use vulkanalia::prelude::v1_0::*;

use crate::infrastructure::image::{create_depth_image, create_texture_image};
use crate::world::automata::Spinner;
use crate::world::uniform::UniformBufferObject;
use crate::world::vertex::{Mat4, Vertex};

#[derive(Debug, Clone)]
pub struct Scene {
    pub automata: Vec<fn(scene_data: &mut SceneData) -> Result<()>>,
    pub scene_data: SceneData,
}

#[derive(Debug, Clone)]
pub struct SceneData {
    pub start: Instant,
    pub uniform_data: UniformBufferObject,
    pub vertex_data: [Vertex; 8],
    pub vertex_indices: [u16; 12],
    pub images: Vec<(vk::Image, vk::ImageView, vk::DeviceMemory)>,
    pub depth_image: Option<(vk::Image, vk::ImageView, vk::DeviceMemory)>,
}

pub fn create_scene(aspect_ratio: f32) -> Scene {
    // initialize scene data
    let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0));

    let view = Mat4::look_at_rh(
        point3(2.0, 2.0, 2.0),
        point3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    );

    #[rustfmt::skip]
    let correction = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, 1.0/2.0, 0.0,
        0.0, 0.0, 1.0/2.0, 1.0,
    );
    let mut proj = correction * cgmath::perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);

    let uniform_data = UniformBufferObject { model, view, proj };

    #[rustfmt::skip]
    let vertex_indices: [u16; 12] = [
        0, 1, 2, 2, 3, 0, 
        4, 5, 6, 6, 7, 4
    ];

    Scene {
        automata: vec![Spinner::work],
        scene_data: SceneData {
            start: Instant::now(),
            uniform_data,
            vertex_data: [
                Vertex::new(vec3(-0.5, -0.5, 0.0), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
                Vertex::new(vec3(0.5, -0.5, 0.0), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
                Vertex::new(vec3(0.5, 0.5, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
                Vertex::new(vec3(-0.5, 0.5, 0.0), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
                Vertex::new(vec3(-0.5, -0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
                Vertex::new(vec3(0.5, -0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
                Vertex::new(vec3(0.5, 0.5, -0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
                Vertex::new(vec3(-0.5, 0.5, -0.5), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
            ],
            vertex_indices,
            images: vec![],
            depth_image: Option::None,
        },
    }
}

pub unsafe fn load_images(
    scene_data: &mut SceneData,
    device: &Device,
    instance: &Instance,
    physical_device: PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) -> Result<()> {
    scene_data.images.push(create_texture_image(
        "resources/texture.png",
        device,
        instance,
        physical_device,
        command_pool,
        graphics_queue,
    )?);

    Ok(())
}

pub unsafe fn load_depth_images(
    scene_data: &mut SceneData,
    device: &Device,
    instance: &Instance,
    physical_device: PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    swapchain_extent: vk::Extent2D,
) -> Result<()> {
    scene_data.depth_image = Option::Some(create_depth_image(
        device,
        instance,
        physical_device,
        command_pool,
        graphics_queue,
        swapchain_extent,
    )?);

    Ok(())
}
