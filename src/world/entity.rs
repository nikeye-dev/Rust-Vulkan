#![allow(dead_code)]
#![allow(unused_variables)]

use crate::world::transform::Transform;

//ToDo: Convert to ECS
pub struct Entity {
    pub id: u32,
    pub name: String,
    pub transform: Transform
}