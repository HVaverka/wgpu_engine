use winit::event_loop::{self, EventLoop};

use crate::{core::{core_app::CoreApp, user_app::UserApp as _}, user_app::app::App};

mod core;
mod gpu;
mod user_app;

fn main() -> Result<(), winit::error::EventLoopError>{
    println!("Hello, world!");

    let my_app = App::init();

    let mut application = CoreApp::new(my_app);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut application)
}
