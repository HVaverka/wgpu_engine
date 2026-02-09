use std::sync::Arc;

use wgpu::Device;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct CoreApp {
    window: Option<Arc<Window>>,
    exit_requested: bool,
}

impl CoreApp {
    pub fn new() -> Self {
        Self {
            window: None,
            exit_requested: false,
        }
    }
}

impl ApplicationHandler for CoreApp {
    // for mobile and wasm - not used, only for window creation
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = create_window(event_loop);

            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
    }

    // emitted after one update
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.exit_requested {
            event_loop.exit();
        }
    }
}

fn create_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let win_attr = Window::default_attributes()
        .with_title("VoxelGame")
        .with_resizable(false)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
    // use Arc.
    Arc::new(
        event_loop
            .create_window(win_attr)
            .expect("create window err."),
    )
}
