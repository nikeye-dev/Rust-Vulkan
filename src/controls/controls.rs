use crate::utils::math::{Vector2, Vector3};

pub trait Controls {
    fn add_input(&mut self, input: Vector3);
    fn add_angular_input(&mut self, input: Vector3);
    fn add_angular_input_2d(&mut self, input: Vector2);
}

