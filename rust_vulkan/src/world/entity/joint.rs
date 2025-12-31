use crate::world::vertex::{Vec2, Vec3, Vertex};

#[derive(Debug, Clone)]
pub struct Joint {
    pub loc: Vertex,
}

impl Joint {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Joint {
            // pubic bone
            loc: Vertex {
                pos: Vec3 { x, y, z },
                color: Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                tex_coord: Vec2 { x: 0.0, y: 0.0 },
            },
        }
    }
}
