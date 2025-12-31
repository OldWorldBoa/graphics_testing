use cgmath::{point3, vec3, Deg, Point3};

use crate::world::vertex::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub fov: Deg<f32>,
    pub eye: Point3<f32>,
    pub center: Point3<f32>,
    pub up: Vec3,
    pub proj: Mat4,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        #[rustfmt::skip]
        let correction = Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0,
            0.0, 0.0, 1.0/2.0, 0.0,
            0.0, 0.0, 1.0/2.0, 1.0,
        );
        let fov = Deg(45.0);
        let proj = correction * cgmath::perspective(Deg(45.0), aspect_ratio, 0.1, 100.0);

        Camera {
            eye: point3(6.0, 0.0, 2.0),
            center: point3(0.0, 0.0, 0.0),
            up: vec3(0.0, 0.0, 1.0),
            fov,
            proj,
        }
    }

    pub fn get_ubo(&self) -> UniformBufferObject {
        UniformBufferObject {
            view: Mat4::look_at_rh(self.eye, self.center, self.up),
            proj: self.proj,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    pub view: Mat4,
    pub proj: Mat4,
}
