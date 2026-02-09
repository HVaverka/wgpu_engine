use std::sync::Arc;

use wgpu::Device;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::core::user_app::UserApp;

pub struct CoreApp<T: UserApp> {
    window: Option<Arc<Window>>,

    user_app: T,
    exit_requested: bool,
}

impl<T: UserApp> CoreApp<T> {
    pub fn new(user_upp: T) -> Self {
        Self {
            window: None,
            user_app: user_upp,
            exit_requested: false,
        }
    }
}

impl<T: UserApp> ApplicationHandler for CoreApp<T> {
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
        .with_title("wGPU Engine")
        .with_resizable(false)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
    // use Arc.
    Arc::new(
        event_loop
            .create_window(win_attr)
            .expect("create window err."),
    )
}
