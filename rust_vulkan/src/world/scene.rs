use anyhow::Result;
use cgmath::{point3, vec2, vec3, Deg};
use std::time::Instant;

use crate::world::automata::Spinner;
use crate::world::uniform::UniformBufferObject;
use crate::world::vertex::{Mat4, Vertex};

#[derive(Debug, Clone)]
pub struct Scene {
    pub start: Instant,
    pub automata: Vec<fn(app_scene: &mut Scene) -> Result<()>>,
    pub uniform_data: UniformBufferObject,
    pub vertex_data: [Vertex; 4],
    pub vertex_indices: [u16; 6],
}

pub fn create_scene(aspect_ratio: f32) -> Scene {
    // initialize scene data
    let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0));

    let view = Mat4::look_at_rh(
        point3(2.0, 2.0, 2.0),
        point3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    );

    let mut proj = cgmath::perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);

    proj[1][1] *= -1.0;

    let uniform_data = UniformBufferObject { model, view, proj };
    let vertex_indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    Scene {
        start: Instant::now(),
        automata: vec![Spinner::work],
        uniform_data,
        vertex_data: [
            Vertex::new(vec2(-0.5, -0.5), vec3(1.0, 0.0, 0.0)),
            Vertex::new(vec2(0.5, -0.5), vec3(0.0, 1.0, 0.0)),
            Vertex::new(vec2(0.5, 0.5), vec3(0.0, 0.0, 1.0)),
            Vertex::new(vec2(-0.5, 0.5), vec3(1.0, 1.0, 1.0)),
        ],
        vertex_indices,
    }
}
