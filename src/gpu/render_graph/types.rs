use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use slotmap::new_key_type;
use wgpu::{BindGroupLayout, CommandEncoder};

new_key_type! {
    pub struct BufferHandle;
    pub struct TextureHandle;
    pub struct PipelineHandle;
}

pub enum NodeType {
    RenderPass,
    ComputePass,
    Transfer,
}

pub struct Node {
    pub name: String,
    pub kind: NodeType,
    pub inputs: Vec<NodeInput>,
    pub outputs: Vec<NodeOutput>,

    pub pipeline: Option<PipelineHandle>,

    pub execute: Option<Box<dyn FnOnce(&mut CommandEncoder)>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResourceHandle {
    Buffer(BufferHandle),
    Texture(TextureHandle),
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct BufferDesc {
    pub size: u64,
    pub usage: wgpu::BufferUsages,
    pub mapped_at_creation: bool,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct TextureDesc {
    size: wgpu::Extent3d,
    // mip_level_count: u32 = 1,
    // sample_count: u32 = 1,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    // view_formats: &'a [] = base is Rgba8SnormSrgb
}
pub struct NodeInput {
    pub binding: u32,
    pub resource: ResourceHandle,
}
pub struct NodeOutput {
    pub binding: u32,
    pub resource: ResourceHandle,
}
#[derive(Hash, PartialEq, Eq)]
pub enum ResourceType {
    Buffer,
    Texture,
}

pub enum PipelineType {
    Render { state: bool },
    Compute,
}

pub struct UploadOp {
    pub target: ResourceHandle,
    pub offset: u64,
    pub data: Vec<u8>,
}

pub struct DownloadOp {
    pub source: ResourceHandle,
    pub offset: u64,
    pub size: u64,
    pub data: Vec<u8>,
}

pub struct CopyOp {
    pub src: ResourceHandle,
    pub dst: ResourceHandle,
    pub size: u64,
    pub src_offset: u64,
    pub dst_offset: u64,
}

pub struct ReadbackTicket<T> {
    data: Arc<Mutex<Option<Vec<u8>>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> ReadbackTicket<T> {
    pub fn try_get(&self) -> Option<T> {
        if let Ok(lock) = self.data.try_lock() {
            lock.as_ref().map(|bytes| *bytemuck::from_bytes::<T>(bytes))
        } else {
            None
        }
    }
}
