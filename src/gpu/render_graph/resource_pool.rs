/*
use std::collections::HashMap;

use slotmap::{DefaultKey, SlotMap};

use crate::gpu::render_graph::types::{PipelineDesc, ResourceDesc};

struct ResourcePool {
    render_pipelines: SlotMap<DefaultKey, wgpu::RenderPipeline>,
    compute_pipelines: SlotMap<DefaultKey, wgpu::ComputePipeline>,

    textures: SlotMap<DefaultKey, wgpu::Texture>,
    buffers: SlotMap<DefaultKey, wgpu::Buffer>,
}
 */