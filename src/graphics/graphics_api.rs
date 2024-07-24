use std::time::Instant;
use anyhow::Result;
use winit::window::Window;

pub trait GraphicsApi {
    fn initialize(&mut self) -> Result<()>;
    fn update(&mut self);
    fn render(&mut self, window: &Window) -> Result<()>;
    fn destroy(&mut self);

    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}
