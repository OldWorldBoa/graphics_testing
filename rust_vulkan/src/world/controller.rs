use cgmath::{point3, InnerSpace, Matrix4, Quaternion, Rotation3, Vector3, Vector4};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::KeyCode,
};

use crate::world::camera::Camera;

#[derive(Clone, Debug)]
pub struct Controller {
    pressed: Vec<KeyCode>,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
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
        let lr_angle = camera.fov * (delta_x / width);
        let ud_angle = camera.fov * (delta_y / height) / 7.0;
        let view_vector = camera.center - camera.eye;

        let y_rotation = Matrix4::from(Quaternion::from_axis_angle(
            view_vector.cross(camera.up),
            ud_angle,
        ));
        let z_rotation = Matrix4::from(Quaternion::from_axis_angle(camera.up, lr_angle));

        let view_vector4 = Vector4::new(view_vector.x, view_vector.y, view_vector.z, 0.0);
        let rotated_vector = z_rotation * y_rotation * view_vector4;

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
        if self.pressed.is_empty() {
            return;
        }

        if self.pressed.contains(&KeyCode::KeyW) {
            move_vector = (camera.center - camera.eye).normalize() / 300.0;
        } else if self.pressed.contains(&KeyCode::KeyS) {
            move_vector = (camera.center - camera.eye).normalize() / 300.0;
            move_vector *= -1.0;
        } else if self.pressed.contains(&KeyCode::KeyA) {
            move_vector = (camera.center - camera.eye).normalize();
            move_vector = (move_vector.cross(camera.up) * -1.0) / 300.0;
        } else if self.pressed.contains(&KeyCode::KeyD) {
            move_vector = (camera.center - camera.eye).normalize();
            move_vector = (move_vector.cross(camera.up)) / 300.0;
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
