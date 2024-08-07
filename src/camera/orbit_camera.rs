use crate::camera::camera::Camera;
use crate::controls::controls::Controls;
use crate::utils::math::{Vector2, Vector3};
use crate::world::transform::OwnedTransform;

#[derive(Debug)]
pub struct OrbitCamera {
    camera: Box<Camera>,
}

impl OrbitCamera {
    pub fn camera(&self) -> &Camera {
        self.camera.as_ref()
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        self.camera.as_mut()
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            camera: Box::new(Camera::default())
        }
    }
}

impl Controls for OrbitCamera {
    fn add_input(&mut self, input: Vector3) {
        let target = self.camera.as_mut();
        let mut location = target.transform().location();

        location += target.transform().inverse_transform_vector(input);
        target.transform_mut().set_location(location);
    }

    fn add_angular_input(&mut self, input: Vector3) {
        let target = self.camera.as_mut();
        target.transform_mut().rotate(input.x, input.y, input.z);
    }

    fn add_angular_input_2d(&mut self, input: Vector2) {
        self.add_angular_input(input.extend(0.0))
    }
}