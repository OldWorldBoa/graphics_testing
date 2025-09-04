use anyhow::Result;
use cgmath::{vec3, Deg};

use crate::world::scene::SceneData;
use crate::world::vertex::Mat4;

/*
* impls in here will be put in the Scene.automata list
*/

pub struct Spinner;
impl Spinner {
    pub fn work(scene_data: &mut SceneData) -> Result<()> {
        let time = scene_data.start.elapsed().as_secs_f32() / 3.0;

        scene_data.uniform_data.model =
            Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0) * time);

        Ok(())
    }
}

pub struct Mover;
impl Mover {
    pub fn work(scene_data: &mut SceneData) -> Result<()> {
        for vert in &mut scene_data.vertex_data {
            vert.pos.x += 1.0;
        }

        Ok(())
    }
}
