use std::time::Instant;
use crate::camera::camera::Camera;
use crate::utils::math::{Vector3, VECTOR3_ONE, Zero};
use crate::world::entity::Entity;
use crate::world::transform::Transform;

pub struct World {
    start_time: Instant,
    main_camera: Camera,
    entities: Vec<Entity>
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

        Self {
            start_time: Instant::now(),
            main_camera: Camera::default(),
            entities
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

    pub fn get_entities(&self) -> Vec<&Entity> {
        self.entities.iter().collect()
    }
}