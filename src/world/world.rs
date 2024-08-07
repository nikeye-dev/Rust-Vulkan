use std::time::Instant;

use crate::camera::camera::Camera;
use crate::camera::orbit_camera::OrbitCamera;
use crate::utils::math::Vector3;
use crate::world::entity::Entity;
use crate::world::transform::{OwnedTransform, Transform};

pub struct World {
    start_time: Instant,
    //ToDo: Make private and handle input
    pub main_camera: OrbitCamera,
    entities: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        let test_entity = Entity {
            id: 1,
            name: "test".into(),
            transform: Transform::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0))
        };
  
        let entities = vec![
            test_entity
        ];

        let orbit_camera = OrbitCamera::default();

        Self {
            start_time: Instant::now(),
            main_camera: orbit_camera,
            entities,
        }
    }

    pub fn active_camera(&self) -> &Camera {
        self.main_camera.camera()
    }

    pub fn active_camera_mut(&mut self) -> &mut Camera {
        self.main_camera.camera_mut()
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    pub fn get_entities(&self) -> Vec<&Entity> {
        self.entities.iter().collect()
    }
}