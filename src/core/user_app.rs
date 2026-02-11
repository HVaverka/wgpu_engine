use crate::gpu::render_graph::{self, graph::RenderGraph};

pub trait UserApp {
    fn init() -> Self;

    fn update(&mut self, render_graph: &mut RenderGraph);
    fn render();
    fn record();
}
