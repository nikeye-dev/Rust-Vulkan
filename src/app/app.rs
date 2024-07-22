use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use anyhow::Result;
use log::info;
use crate::config::config::{Config, GraphicsApiType};
use crate::graphics::graphics_api::GraphicsApi;
use crate::graphics::vulkan::vulkan_api::VulkanApi;

pub struct App {
    config: Config,
    window: Option<Window>,
    graphics: Option<VulkanApi>,
    start_time: Instant
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "Rust - Vulkan".to_string();

        self.window = Some(event_loop.create_window(attributes).unwrap());

        if self.graphics.is_none() {
            info!("Creating graphics...");
            let mut api = VulkanApi::new(self.window.as_ref().unwrap(), self.config.graphics.get(&GraphicsApiType::Vulkan).cloned().unwrap());
            api.initialize().unwrap();

            self.graphics = Some(api);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.
                self.render().unwrap();

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("Destroying app");

        match self.graphics.as_mut() {
            Some(x) => x.destroy(),
            None => ()
        }
    }
}

impl App {
    pub(crate) fn new(config: Config) -> Self {
        Self { config, window: None, graphics: None, start_time: Instant::now() }
    }

    pub(crate) fn run(&mut self) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)
    }

    fn render(&mut self) -> Result<()> {
        if self.graphics.is_some() {
            return self.graphics.as_mut().unwrap().render(self.window.as_ref().unwrap(), self.start_time);
        }

        Ok(())
    }
}

