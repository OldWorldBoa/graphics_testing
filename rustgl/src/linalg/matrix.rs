use std::ops::Mul;

pub fn zeros(height: u32, width: u32) -> Matrix {
    let mut mat = Matrix {
        height,
        width,
        data: vec![],
    };

    for _ in 0..height {
        mat.data.push(vec![0.0; width as usize]);
    }

    mat
}

pub fn identity() -> Matrix {
    Matrix {
        height: 4,
        width: 4,
        data: vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ],
    }
}

pub struct Matrix {
    height: u32,
    width: u32,
    pub data: Vec<Vec<f32>>,
}

impl Matrix {
    fn add_translate(&mut self, vec: Vec<f32>) {
        assert_eq!(3, vec.len());
        self.data[0][4] = vec[0];
        self.data[1][4] = vec[1];
        self.data[2][4] = vec[2];
    }

    fn raw(self) -> Vec<f32> {
        let mut raw = vec![];
        for i in 0..self.height {
            for j in 0..self.width {
                raw.push(self.data[i as usize][j as usize]);
            }
        }

        raw
    }
}

impl Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.width, rhs.height);
        let mut mat = self::zeros(self.height, rhs.width);

        for i in 0..mat.height {
            for j in 0..mat.width {
                let mut curr_item = 0.0;
                for x in 0..self.width {
                    curr_item +=
                        self.data[i as usize][x as usize] * rhs.data[x as usize][j as usize];
                }

                mat.data[i as usize][j as usize] = curr_item;
            }
        }

        return mat;
    }
}

