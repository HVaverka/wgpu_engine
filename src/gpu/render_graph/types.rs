use std::path::{Path, PathBuf};

use wgpu::{BindGroupLayout, CommandEncoder};

#[derive(Clone, Copy)]
pub struct ResourceHandle(pub u32);
#[derive(Clone, Copy)]
pub struct PipelineHandle(pub u32);


pub enum PassType {
    RenderPass,
    ComputePass,
    Transfer,
}

pub struct Pass {
    pub name: String,
    pub kind: PassType,
    pub inputs: Vec<PassInput>,
    pub outputs: Vec<PassOutput>,

    pub pipeline: Option<PipelineHandle>,

    pub execute: Box<dyn FnOnce(&mut CommandEncoder)>
}

pub struct PassInput {
    pub binding: u32,
    pub resource: ResourceHandle,
}
pub struct PassOutput {
    pub binding: u32,
    pub resource: ResourceHandle,
}
pub enum ResourceType {
    Buffer,
    Texture,
}

pub enum PipelineType {
    Render{
        state: bool,
    },
    Compute,
}
pub struct ResourceDesc {
    name: String,
    kind: ResourceType,
    size: u64,
    dimensions: (u32, u32),
    is_persistent: bool,
}

pub struct PipelineDesc {
    name: String,
    shader_path: PathBuf,
    entry_point: String,
    kind: PipelineType,
}