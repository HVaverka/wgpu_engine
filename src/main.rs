use winit::event_loop::{self, EventLoop};

use crate::core::core_app::CoreApp;

mod core;

fn main() -> Result<(), winit::error::EventLoopError>{
    println!("Hello, world!");

    let mut application = CoreApp::new();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut application)
}
