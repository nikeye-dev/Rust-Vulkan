pub trait GameObject {
    fn start(&mut self) {}
    fn update(&mut self, delta_time: f32) {}
    fn destroy(&mut self) {}
}