use std::ops::Add;
use cgmath::InnerSpace;
use crate::utils::math::{Deg, Euler, Matrix4x4, Quaternion, Vector3, Zero};

#[derive(Debug)]
pub struct Transform {
    location: Vector3,
    rotation: Quaternion,
    scale: Vector3
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            location: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0)
        }
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self::default()
    }

    pub fn location(&self) -> Vector3 {
        self.location
    }

    pub fn rotation(&self) -> Quaternion {
        self.rotation
    }

    pub fn scale(&self) -> Vector3 {
        self.scale
    }

    pub fn matrix(&self) -> Matrix4x4 {
        let t = Matrix4x4::from_translation(self.location);
        let r = Matrix4x4::from(self.rotation.normalize());
        let s = Matrix4x4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        t * r * s
    }

    pub fn set_location(&mut self, new_location: Vector3) {
        self.location = new_location;
    }

    pub fn set_location_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.location = Vector3::new(x, y, z)
    }

    pub fn set_rotation_euler_deg(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = Quaternion::from(Euler {
            x: Deg(x),
            y: Deg(y),
            z: Deg(z)
        })
    }

    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = Vector3::new(x, y, z);
    }

    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.set_scale(scale, scale, scale);
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        let add = Quaternion::from(Euler {
            x: Deg(x),
            y: Deg(y),
            z: Deg(z)
        });

        self.rotation += add;
    }
}

pub trait OwnedTransform {
    fn transform(&self) -> &Transform;

    fn transform_mut(&mut self) -> &mut Transform;
}
