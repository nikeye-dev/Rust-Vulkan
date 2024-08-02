use std::time::Instant;
use crate::camera::camera::Camera;

pub struct World {
    start_time: Instant,
    main_camera: Camera,
}

impl World {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            main_camera: Camera::default()
        }
    }

    pub fn active_camera(&self) -> &Camera {
        &self.main_camera
    }

    pub fn active_camera_mut(&mut self) -> &mut Camera {
        &mut self.main_camera
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}