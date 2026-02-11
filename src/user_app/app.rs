use crate::gpu::render_graph::types::*;
use crate::{
    core::user_app::UserApp,
    gpu::render_graph::{self, graph::RenderGraph},
};
pub struct App {}

impl UserApp for App {
    fn init() -> Self {
        App {}
    }

    fn update(&mut self, render_graph: &mut RenderGraph) {}

    fn render() {}

    fn record() {}
}
