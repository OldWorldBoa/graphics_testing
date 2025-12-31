use anyhow::Result;
use cgmath::{vec3, InnerSpace, Vector3};

use crate::{
    infrastructure::image::load_texture_image,
    world::{
        entity::{entity::Entity, joint::Joint},
        vertex::{Mat4, Vec2, Vec3, Vertex},
    },
};

#[derive(Debug, Clone)]
pub struct Human {
    pub joints: Vec<Joint>,
    pub edges: Vec<(u32, u32)>,
    pub entity: Entity,
}

impl Default for Human {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Human {
    pub fn new() -> Result<Self> {
        let mut human = Human {
            joints: vec![
                Joint::new(0.0, 0.0, 0.0),    // Pubic joint
                Joint::new(-2.0, 0.0, 0.0),   // R Hip
                Joint::new(-2.0, -7.0, 0.0),  // R Knee
                Joint::new(-2.0, -13.0, 0.0), // R Ankle
                Joint::new(2.0, 0.0, 0.0),    // L Hip
                Joint::new(2.0, -7.0, 0.0),   // L Knee
                Joint::new(2.0, -13.0, 0.0),  // L Ankle
                Joint::new(0.0, 11.0, 0.0),   // Collar
                Joint::new(0.0, 15.0, 0.0),   // Head
                Joint::new(-4.0, 11.0, 0.0),  // R Shoulder
                Joint::new(-4.0, 4.0, 0.0),   // R Elbow
                Joint::new(-4.0, -2.0, 0.0),  // R Wrist
                Joint::new(4.0, 11.0, 0.0),   // L Shoulder
                Joint::new(4.0, 4.0, 0.0),    // L Shoulder
                Joint::new(4.0, -2.0, 0.0),   // L Shoulder
            ],
            edges: vec![
                (0, 1),
                (0, 4),
                (0, 7),
                (1, 2),
                (2, 3),
                (4, 5),
                (5, 6),
                (7, 8),
                (7, 9),
                (7, 12),
                (9, 10),
                (10, 11),
                (12, 13),
                (13, 14),
            ],
            entity: Entity {
                vertex_data: vec![],
                vertex_indices: vec![],
                transform: Mat4::from_translation(vec3(0.0, 1.25, 1.0)) * Mat4::from_scale(0.1),
                texture: load_texture_image("resources/texture.png")?,
                opacity: 1.0,
                mip_levels: 1,
            },
        };

        human.skin()?;

        Ok(human)
    }

    fn skin(self: &mut Self) -> Result<()> {
        for edge in self.edges.iter() {
            let origin = &self.joints[edge.0 as usize];
            let dest = &self.joints[edge.1 as usize];

            let (vertices, indices) = connect_vertices(origin.loc, dest.loc);
            let idx_base = self.entity.vertex_data.len();

            for vertex in vertices.iter() {
                self.entity.vertex_data.push(*vertex);
            }

            for index in indices.iter() {
                self.entity.vertex_indices.push(idx_base as u32 + *index);
            }
        }

        Ok(())
    }
}

fn connect_vertices(origin: Vertex, dest: Vertex) -> (Vec<Vertex>, Vec<u32>) {
    let radius = 0.2;

    let vector = dest.pos - origin.pos;
    let mut cross = vector
        .cross(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        })
        .normalize();
    cross *= radius;

    (
        vec![
            Vertex::new(
                origin.pos + cross,
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
            Vertex::new(
                origin.pos - cross,
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
            Vertex::new(
                dest.pos + cross,
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
            Vertex::new(
                dest.pos - cross,
                Vec3::new(1.0, 1.0, 1.0),
                Vec2::new(1.0, 1.0),
            ),
        ],
        vec![0, 1, 2, 1, 0, 2, 1, 2, 3, 2, 1, 3],
    )
}
