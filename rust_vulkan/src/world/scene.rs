use anyhow::Result;
use cgmath::{vec2, vec3};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use crate::world::automata::{Mover, Spinner};
use crate::world::camera::Camera;
use crate::world::vertex::{Mat4, Vertex};

#[derive(Debug, Clone)]
pub struct Scene {
    // Automata
    pub spinner: Spinner,
    pub mover: Mover,
    pub scene_data: SceneData,
}
impl Scene {
    pub fn create_scene(aspect_ratio: f32) -> Result<Self> {
        Ok(Scene {
            spinner: Spinner { last_time: 0f32 },
            mover: Mover,
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
pub struct Entity {
    pub transform: Mat4,
    pub vertex_data: Vec<Vertex>,
    pub vertex_indices: Vec<u32>,
    pub texture: Texture,
    pub opacity: f32,
    pub mip_levels: u32,
}

#[derive(Debug, Clone)]
pub struct SceneData {
    pub start: Instant,
    pub camera: Camera,
    pub entities: Vec<Entity>,
}
impl SceneData {
    pub fn create_scene_data(aspect_ratio: f32) -> Result<Self> {
        // initialize scene data
        let mut scene_data = SceneData {
            start: Instant::now(),
            camera: Camera::new(aspect_ratio),
            entities: vec![],
        };

        scene_data.load_entities()?;

        Ok(scene_data)
    }

    pub fn load_entities(&mut self) -> Result<()> {
        let (vertex_data, vertex_indices) = self.load_model("resources/viking_room.obj")?;
        self.entities.push(Entity {
            transform: Mat4::from_translation(vec3(0.0, -1.25, 1.0)),
            vertex_data,
            vertex_indices,
            texture: self.load_texture_image("resources/viking_room.png")?,
            opacity: 0.25,
            mip_levels: 1,
        });

        let (vertex_data, vertex_indices) = self.load_model("resources/viking_room.obj")?;
        self.entities.push(Entity {
            transform: Mat4::from_translation(vec3(0.0, -1.25, -1.0)),
            vertex_data,
            vertex_indices,
            texture: self.load_texture_image("resources/viking_room.png")?,
            opacity: 0.25,
            mip_levels: 1,
        });

        let (vertex_data, vertex_indices) = self.load_model("resources/viking_room.obj")?;
        self.entities.push(Entity {
            transform: Mat4::from_translation(vec3(0.0, 1.25, 0.0)),
            vertex_data,
            vertex_indices,
            texture: self.load_texture_image("resources/viking_room.png")?,
            opacity: 0.25,
            mip_levels: 1,
        });

        Ok(())
    }

    /// load scene images
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub fn load_texture_image(&self, path: &str) -> Result<Texture> {
        let image = File::open(path)?;

        let decoder = png::Decoder::new(image);
        let mut reader = decoder.read_info()?;
        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;

        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();

        Ok(Texture {
            pixels,
            size,
            height,
            width,
        })
    }

    /// load scene model
    ///
    /// # Safety
    /// Check the vulkan docs for safety info
    pub fn load_model(&self, path: &str) -> Result<(Vec<Vertex>, Vec<u32>)> {
        let mut reader = BufReader::new(File::open(path)?);
        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
            |_| Ok(Default::default()),
        )?;

        let mut unique_vertices = HashMap::new();
        let mut vertex_data = vec![];
        let mut vertex_indices = vec![];

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
                    vertex_indices.push(*index as u32);
                } else {
                    let index = vertex_data.len();
                    unique_vertices.insert(vertex, index);
                    vertex_data.push(vertex);
                    vertex_indices.push(index as u32);
                }
            }
        }

        Ok((vertex_data, vertex_indices))
    }
}
