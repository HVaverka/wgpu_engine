use wgpu::RenderPipelineDescriptor;

use crate::{
    core::user_app::UserApp,
    gpu::render_graph::{self, graph::RenderGraph, resource_pool::{RenderPipelineDesc, Resources, VertexState}, types::RenderPipelineHandle},
};

pub struct App {
    render_pipeline: RenderPipelineHandle,
}

impl UserApp for App {
    fn init(resources: &mut Resources) -> Self {
        let shader = resources.load_shader("base");
        let pipeline_layout = resources.add_pipeline_layout(
            wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                immediate_size: 0,
            }
        );

        let render_pipeline = resources.create_render_pipeline(
            RenderPipelineDesc {
                label: Some("Render pipeline"),
                layout: pipeline_layout,
                vertex: VertexState {
                    module: shader,
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 0,
                    mask: 0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: None,
                multiview_mask: None,
            }
        );

        App {
            render_pipeline,
        }
    }

    fn update(&mut self, render_graph: &mut RenderGraph) {}

    fn render() {}

    fn record() {}
}
