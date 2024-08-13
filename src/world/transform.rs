use cgmath::{InnerSpace, One, Rotation3, Transform as cgTransform};

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
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0)
        }
    }
}

impl Transform {
    pub fn new(location: Vector3, rotation: Vector3, scale: Vector3) -> Self {
        Transform {
            location,
            rotation: Quaternion::from(Euler {
                x: Deg(rotation.x),
                y: Deg(rotation.y),
                z: Deg(rotation.z),
            }).normalize(),
            scale
        }
    }

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
        // self.matrix_t() * self.matrix_r() * self.matrix_s()
        self.matrix_s() * self.matrix_r() * self.matrix_t()
    }

    pub fn matrix_t(&self) -> Matrix4x4 {
        Matrix4x4::from_translation(self.location)
    }

    pub fn matrix_r(&self) -> Matrix4x4 {
        let rot = self.rotation;

        let xx2 = 2.0 * rot.v.x * rot.v.x;
        let yy2 = 2.0 * rot.v.y * rot.v.y;
        let zz2 = 2.0 * rot.v.z * rot.v.z;

        let xy2 = 2.0 * rot.v.x * rot.v.y;
        let xz2 = 2.0 * rot.v.x * rot.v.z;
        let yz2 = 2.0 * rot.v.y * rot.v.z;

        let wx2 = 2.0 * rot.v.x * rot.s;
        let wy2 = 2.0 * rot.v.y * rot.s;
        let wz2 = 2.0 * rot.v.z * rot.s;

        Matrix4x4::new(
            1.0 - yy2 - zz2, xy2 - wz2, xz2 + wy2, 0.0,
            xy2 + wz2, 1.0 - xx2 - zz2, yz2 - wx2, 0.0,
            xz2 - wy2, yz2 + wx2, 1.0 - xx2 - yy2, 0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn matrix_s(&self) -> Matrix4x4 {
        Matrix4x4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
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
        }).normalize()
    }

    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = Vector3::new(x, y, z);
    }

    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.set_scale(scale, scale, scale);
    }

    pub fn rotate_vec(&mut self, rot: Vector3) {
        self.rotate(rot.x, rot.y, rot.z);
    }

    pub fn rotate(&mut self, x_deg: f32, y_deg: f32, z_deg: f32) {
        let pitch = Quaternion::from_angle_x(Deg(x_deg));
        let yaw = Quaternion::from_angle_y(Deg(y_deg));
        let roll = Quaternion::from_angle_z(Deg(z_deg));

        self.rotation = (yaw * self.rotation * pitch) * roll;
    }

    pub fn transform_vector(&self, vector: Vector3) -> Vector3 {
        self.matrix().transform_vector(vector)
    }

    pub fn inverse_transform_vector(&self, vector: Vector3) -> Vector3 {
        self.matrix().inverse_transform_vector(vector).unwrap()
    }
}

pub trait OwnedTransform {
    fn transform(&self) -> &Transform;

    fn transform_mut(&mut self) -> &mut Transform;
}
