use std::ops::Index;

use cgmath::{point3, InnerSpace, Vector3, Vector4};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, ModifiersKeyState},
};

use crate::world::{camera::Camera, vertex::Mat4};

#[derive(Clone, Debug)]
pub struct Controller {
    pressed: Vec<KeyCode>,
}

impl Controller {
    pub fn new() -> Self {
        Controller { pressed: vec![] }
    }

    pub fn handle_camera_move(
        &self,
        camera: &mut Camera,
        motion: (f32, f32),
        height: f32,
        width: f32,
    ) {
        let delta_x = -motion.0;
        let delta_y = -motion.1;

        let x_angle = camera.fov * (delta_x / width);
        let y_angle = camera.fov * (delta_y / height);

        let view_vector = camera.center - camera.eye;
        let view_vector4 = Vector4::new(view_vector.x, view_vector.y, view_vector.z, 0.0);
        let z_rotation = Mat4::from_angle_z(x_angle);
        let y_rotation = Mat4::from_angle_y(y_angle);
        let rotated_vector = y_rotation * z_rotation * view_vector4;

        camera.center = point3(
            rotated_vector.x + camera.eye.x,
            rotated_vector.y + camera.eye.y,
            rotated_vector.z + camera.eye.z,
        );
    }

    pub fn handle_keypress(&mut self, key_event: KeyEvent) {
        match key_event.physical_key {
            winit::keyboard::PhysicalKey::Code(key_code) => {
                if self.pressed.contains(&key_code) && key_event.state == ElementState::Released {
                    let index = self.pressed.iter().position(|r| r == &key_code).unwrap();
                    self.pressed.remove(index);
                } else if !self.pressed.contains(&key_code)
                    && key_event.state == ElementState::Pressed
                {
                    self.pressed.push(key_code);
                }
            }
            winit::keyboard::PhysicalKey::Unidentified(native_key_code) => {
                println!("Unknown key code {:?}", native_key_code);
            }
        }
    }

    pub fn process_key_commands(&self, camera: &mut Camera) {
        let mut move_vector = Vector3::new(0.0, 0.0, 0.0);
        if self.pressed.len() == 0 {
            return;
        }

        if self.pressed.contains(&KeyCode::KeyW) {
            move_vector = (camera.center - camera.eye).normalize() / 300.0;
        } else if self.pressed.contains(&KeyCode::KeyS) {
            move_vector = (camera.center - camera.eye).normalize() / 300.0;
            move_vector *= -1.0;
        } else if self.pressed.contains(&KeyCode::KeyA) {
            move_vector = (camera.center - camera.eye).normalize() / 300.0;
            move_vector = move_vector.cross(camera.up);
        } else if self.pressed.contains(&KeyCode::KeyD) {
            move_vector = (camera.center - camera.eye).normalize() / 300.0;
            move_vector = (move_vector.cross(camera.up)) * -1.0;
        }

        camera.eye = point3(
            camera.eye.x + move_vector.x,
            camera.eye.y + move_vector.y,
            camera.eye.z + move_vector.z,
        );

        camera.center = point3(
            camera.center.x + move_vector.x,
            camera.center.y + move_vector.y,
            camera.center.z + move_vector.z,
        );
    }
}
