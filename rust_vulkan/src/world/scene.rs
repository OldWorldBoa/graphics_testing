use anyhow::Result;
use cgmath::{point3, vec2, vec3, Deg};
use rand::{rng, thread_rng};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use crate::world::automata::{Mover, Spinner};
use crate::world::uniform::UniformBufferObject;
use crate::world::vertex::{Mat4, Vertex};

#[derive(Debug, Clone)]
pub struct Scene {
    pub automata: Vec<fn(scene_data: &mut SceneData) -> Result<()>>,
    pub scene_data: SceneData,
}
impl Scene {
    pub fn create_scene(aspect_ratio: f32) -> Result<Scene> {
        Ok(Scene {
            automata: vec![Spinner::work, Mover::work],
            scene_data: SceneData::create_scene_data(aspect_ratio)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub pixels: Vec<u8>,
    pub size: u64,
    pub height: u32,
    pub width: u32,
}

#[derive(Debug, Clone)]
pub struct SceneData {
    pub start: Instant,
    pub uniform_data: UniformBufferObject,
    pub opacity: f32,
    pub model: Mat4,
    pub vertex_data: Vec<Vertex>,
    pub vertex_indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub mip_levels: u32,
}
impl SceneData {
    pub fn create_scene_data(aspect_ratio: f32) -> Result<Self> {
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

        let uniform_data = UniformBufferObject { view, proj };

        let mut scene_data = SceneData {
            start: Instant::now(),
            uniform_data,
            model,
            opacity: 0.25,
            vertex_data: vec![],
            vertex_indices: vec![],
            textures: vec![],
            mip_levels: 1,
        };

        scene_data.load_texture_images()?;
        scene_data.load_model()?;

        Ok(scene_data)
    }

    /// load scene images
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub fn load_texture_images(&mut self) -> Result<()> {
        let image = File::open("resources/viking_room.png")?;

        let decoder = png::Decoder::new(image);
        let mut reader = decoder.read_info()?;
        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;

        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();

        self.textures.push(Texture {
            pixels,
            size,
            height,
            width,
        });

        Ok(())
    }

    /// load scene model
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub fn load_model(&mut self) -> Result<()> {
        let mut reader = BufReader::new(File::open("resources/viking_room.obj")?);

        self.vertex_data.clear();
        self.vertex_indices.clear();

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
                    self.vertex_indices.push(*index as u32);
                } else {
                    let index = self.vertex_data.len();
                    unique_vertices.insert(vertex, index);
                    self.vertex_data.push(vertex);
                    self.vertex_indices.push(index as u32);
                }
            }
        }

        Ok(())
    }
}
