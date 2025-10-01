use cgmath::{vec3, Deg, Transform};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::world::scene::SceneData;
use crate::world::vertex::Mat4;

/*
* impls in here will be put in the Scene.automata list
*/
#[derive(Clone, Debug)]
pub struct Spinner {
    pub last_time: f32,
}
impl Spinner {
    pub fn work(&mut self, scene_data: &mut SceneData) {
        let time = scene_data.start.elapsed().as_secs_f32();
        let delta = time - self.last_time;

        for entity in scene_data.entities.iter_mut() {
            entity.transform =
                entity.transform * Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(10.0) * delta);
        }

        self.last_time = time;
    }
}

#[derive(Clone, Debug)]
pub struct Mover;
impl Mover {
    pub fn work(&mut self, scene_data: &mut SceneData) {
        for entity in scene_data.entities.iter_mut() {
            let time = scene_data.start.elapsed().as_secs_f32();
            entity.transform =
                entity.transform * Mat4::from_translation(vec3(0.0, time.sin() / 10000.0, 0.0));
        }
    }
}
