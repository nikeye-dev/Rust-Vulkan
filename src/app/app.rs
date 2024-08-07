use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use anyhow::Result;
use log::{debug, info};
use crate::config::config::{Config, GraphicsApiType};
use crate::graphics::rhi::RHI;
use crate::graphics::vulkan::vulkan_rhi::RHIVulkan;
use crate::world::transform::OwnedTransform;
use crate::world::world::World;

pub struct App {
    config: Config,
    window: Option<Window>,
    graphics: Option<RHIVulkan>,
    world_ref: Arc<RwLock<World>>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "Rust - Vulkan".to_string();

        self.window = Some(event_loop.create_window(attributes).unwrap());

        if self.graphics.is_none() {
            info!("Creating graphics...");
            let mut api = RHIVulkan::new(self.window.as_ref().unwrap(), self.config.graphics.get(&GraphicsApiType::Vulkan).cloned().unwrap());
            api.initialize(self.world_ref.clone()).unwrap();

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
            },
            _ => (),
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion {delta} => {
                let mut world = self.world_ref.write().unwrap();

                let (x, y) = (delta.0.clamp(-1.0, 1.0), delta.1.clamp(-1.0, 1.0));
                debug!("Mouse delta(mod): {:?}, {:?}", x, y);

                world.active_camera_mut().transform_mut().rotate(y as f32, x as f32, 0.0);
            },
            _ => ()
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
        let mut world = World::new();
        world.active_camera_mut().transform_mut().set_location_xyz(0.0, 0.0, -5.0);
        // world.active_camera_mut().transform_mut().set_rotation_euler_deg(0.0, 15.0, 0.0);

        Self {
            config,
            window: None,
            graphics: None,
            world_ref: Arc::new(RwLock::new(world))
        }
    }

    pub(crate) fn run(&mut self) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)
    }

    fn render(&mut self) -> Result<()> {
        if self.graphics.is_some() {
            return self.graphics.as_mut().unwrap().render(self.window.as_ref().unwrap());
        }

        Ok(())
    }
}

