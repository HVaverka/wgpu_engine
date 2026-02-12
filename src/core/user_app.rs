use crate::gpu::render_graph::{self, graph::RenderGraph, resource_pool::Resources};

pub trait UserApp {
    fn init(resources: &mut Resources) -> Self;

    fn update(&mut self, render_graph: &mut RenderGraph);
    fn render();
    fn record();
}
