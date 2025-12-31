use crate::world::{
    scene::Texture,
    vertex::{Mat4, Vertex},
};

#[derive(Debug, Clone)]
pub struct Entity {
    pub transform: Mat4,
    pub vertex_data: Vec<Vertex>,
    pub vertex_indices: Vec<u32>,
    pub texture: Texture,
    pub opacity: f32,
    pub mip_levels: u32,
}
