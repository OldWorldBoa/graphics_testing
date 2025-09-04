use anyhow::Result;
use cgmath::{point3, vec2, vec3, Deg};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use vk::PhysicalDevice;
use vulkanalia::prelude::v1_0::*;

use crate::infrastructure::image::{
    create_depth_image, create_sampling_image, create_texture_image, ImageBundle,
};
use crate::infrastructure::swapchain::SwapchainInfo;
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
    pub vertex_data: Vec<Vertex>,
    pub vertex_indices: Vec<u32>,
    pub mip_levels: u32,
    pub images: Vec<ImageBundle>,
    pub sampling_image: Option<ImageBundle>,
    pub depth_image: Option<ImageBundle>,
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
    let proj = correction * cgmath::perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);

    let uniform_data = UniformBufferObject { model, view, proj };

    Scene {
        automata: vec![Spinner::work],
        scene_data: SceneData {
            start: Instant::now(),
            uniform_data,
            vertex_data: vec![],
            vertex_indices: vec![],
            mip_levels: 1,
            images: vec![],
            sampling_image: Option::None,
            depth_image: Option::None,
        },
    }
}

impl Scene {
    /// load scene images
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn load_images(
        &mut self,
        device: &Device,
        instance: &Instance,
        physical_device: PhysicalDevice,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
    ) -> Result<()> {
        self.scene_data.images.push(create_texture_image(
            "resources/viking_room.png",
            device,
            instance,
            physical_device,
            command_pool,
            graphics_queue,
        )?);

        Ok(())
    }

    /// load scene model
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn load_model(&mut self) -> Result<()> {
        let mut reader = BufReader::new(File::open("resources/viking_room.obj")?);

        self.scene_data.vertex_data.clear();
        self.scene_data.vertex_indices.clear();

        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
            |_| Ok(Default::default()),
        )?;

        let mut unique_vertices = HashMap::new();

        for model in &models {
            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let tex_coord_offset = (2 * index) as usize;
                let vertex = Vertex::new(
                    vec3(
                        model.mesh.positions[pos_offset],
                        model.mesh.positions[pos_offset + 1],
                        model.mesh.positions[pos_offset + 2],
                    ),
                    vec3(1.0, 1.0, 1.0),
                    vec2(
                        model.mesh.texcoords[tex_coord_offset],
                        1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                    ),
                );

                if let Some(index) = unique_vertices.get(&vertex) {
                    self.scene_data.vertex_indices.push(*index as u32);
                } else {
                    let index = self.scene_data.vertex_data.len();
                    unique_vertices.insert(vertex, index);
                    self.scene_data.vertex_data.push(vertex);
                    self.scene_data.vertex_indices.push(index as u32);
                }
            }
        }

        Ok(())
    }

    /// load depth images
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn load_depth_image(
        &mut self,
        device: &Device,
        instance: &Instance,
        physical_device: PhysicalDevice,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        swapchain_extent: vk::Extent2D,
    ) -> Result<()> {
        self.scene_data.depth_image = Option::Some(create_depth_image(
            device,
            instance,
            physical_device,
            command_pool,
            graphics_queue,
            swapchain_extent,
        )?);

        Ok(())
    }

    /// load color sample images
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub unsafe fn load_sampling_image(
        &mut self,
        device: &Device,
        instance: &Instance,
        physical_device: PhysicalDevice,
        swapchain: &SwapchainInfo,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<()> {
        self.scene_data.sampling_image = Option::Some(create_sampling_image(
            instance,
            device,
            physical_device,
            swapchain,
            msaa_samples,
        )?);

        Ok(())
    }
}
