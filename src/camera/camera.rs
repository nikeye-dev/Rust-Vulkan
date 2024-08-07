use cgmath::{EuclideanSpace, InnerSpace, Matrix, point3, Point3, SquareMatrix, Transform as tf, vec3};
use log::debug;
use crate::utils::math::{Matrix3x3, Matrix4x4, Quaternion, Vector3, Zero};
use crate::world::transform::{OwnedTransform, Transform};

#[derive(Default, Debug, Copy, Clone)]
pub struct ViewSettings {
    pub near: f32,
    pub far: f32,
    pub fov: f32
}

#[derive(Debug)]
pub struct Camera {
    view_settings: ViewSettings,
    transform: Transform
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_settings: ViewSettings { near: 0.1, far: 10000000000.0, fov: 80.0 },
            transform: Transform::identity()
        }
    }
}

impl OwnedTransform for Camera {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

impl Camera {
    pub fn view(&self) -> ViewSettings {
        self.view_settings
    }

    pub fn view_matrix(&self) -> Matrix4x4 {
        let t = self.transform.matrix_t();
        let r = self.transform.matrix_r();

        //ToDo: make Z go inwards instead of outwards
        r * t
    }
}
