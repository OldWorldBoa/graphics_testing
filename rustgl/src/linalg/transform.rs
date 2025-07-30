use crate::linalg::matrix::{identity, Matrix};

pub struct Transform {
    translate: Matrix,
    scale: Matrix,
    rotate: Matrix,
}

pub fn blank() -> Transform {
    Transform {
        translate: identity(),
        scale: identity(),
        rotate: identity(),
    }
}

impl Transform {
    fn translate(&mut self, vec: Vec<f32>) {
        assert_eq!(3, vec.len());
        self.translate = identity();

        for i in 0..3 {
            self.translate.data[i][4] = vec[i];
        }
    }
}
