use cgmath::InnerSpace;
use log::debug;
use winit::event::{DeviceEvent, ElementState};
use winit::keyboard::{Key, KeyCode, NamedKey};
use crate::camera::camera::Camera;
use crate::controls::controls::Controls;
use crate::utils::math::{Vector2, Vector3, Zero};
use crate::world::game_object::GameObject;
use crate::world::transform::OwnedTransform;

#[derive(Debug)]
pub struct OrbitCamera {
    move_speed: f32,
    rotate_speed: f32,
    camera: Box<Camera>,
    current_input: Vector3,
    current_angular_input: Vector3
}

impl OrbitCamera {
    pub fn camera(&self) -> &Camera {
        self.camera.as_ref()
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        self.camera.as_mut()
    }

    //ToDo: Add more abstract input handling elsewhere
    pub fn handle_input_key(&mut self, key_code: KeyCode, state: ElementState) {

        if state == ElementState::Pressed {
            match key_code {
                KeyCode::KeyW => {
                    self.add_input(Vector3::new(0.0, 0.0, 1.0));
                },
                KeyCode::KeyS => {
                    self.add_input(Vector3::new(0.0, 0.0, -1.0));
                },
                KeyCode::KeyA => {
                    //ToDo: why flipped?
                    self.add_input(Vector3::new(1.0, 0.0, 0.0));
                },
                KeyCode::KeyD => {
                    //ToDo: why flipped?
                    self.add_input(Vector3::new(-1.0, 0.0, 0.0));
                },
                KeyCode::Space => {
                    //ToDo: why flipped?
                    self.add_input(Vector3::new(0.0, -1.0, 0.0));
                },
                KeyCode::ControlLeft => {
                    //ToDo: why flipped?
                    self.add_input(Vector3::new(0.0, 1.0, 0.0));
                },
                _ => ()
            }
        }
        else {
            match key_code {
                KeyCode::KeyW => {
                    self.current_input.z = 0.0;
                },
                KeyCode::KeyS => {
                    self.current_input.z = 0.0;
                },
                KeyCode::KeyA => {
                    self.current_input.x = 0.0;
                },
                KeyCode::KeyD => {
                    self.current_input.x = 0.0;
                },
                KeyCode::Space => {
                    self.current_input.y = 0.0;
                },
                KeyCode::ControlLeft => {
                    self.current_input.y = 0.0;
                },
                _ => ()
            }
        }

    }

    pub fn handle_mouse_move(&mut self, delta: (f64, f64)) {
        let (x, y) = (delta.0.clamp(-1.0, 1.0), delta.1.clamp(-1.0, 1.0));
        self.add_angular_input_2d(Vector2::new(y as f32, x as f32));
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            move_speed: 10.0,
            rotate_speed: 280.0,
            camera: Box::new(Camera::default()),
            current_input: Vector3::zero(),
            current_angular_input: Vector3::zero()
        }
    }
}

impl Controls for OrbitCamera {
    fn add_input(&mut self, input: Vector3) {
        self.current_input = (self.current_input + input).normalize();
    }

    fn add_angular_input(&mut self, input: Vector3) {
        self.current_angular_input += input;
    }

    fn add_angular_input_2d(&mut self, input: Vector2) {
        self.add_angular_input(input.extend(0.0))
    }
}

impl GameObject for OrbitCamera {
    fn update(&mut self, delta_time: f32) {
        if self.current_input.magnitude2() > 0.0 {
            let mut location = self.camera.transform().location();
            location += self.camera.transform().inverse_transform_vector(self.current_input) * self.move_speed * delta_time;

            self.camera_mut().transform_mut().set_location(location);
        }

        if self.current_angular_input.magnitude2() > 0.0 {
            let rotation = self.current_angular_input * self.rotate_speed * delta_time;
            self.camera_mut().transform_mut().rotate_vec(rotation);

            self.current_angular_input = Vector3::zero();
        }
    }
}
