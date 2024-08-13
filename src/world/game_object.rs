#![allow(dead_code)]
#![allow(unused_variables)]

pub trait GameObject {
    fn start(&mut self) {}
    fn update(&mut self, delta_time: f32) {}
    fn destroy(&mut self) {}
}