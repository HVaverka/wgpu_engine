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

use std::num::NonZeroU32;
use std::sync::Arc;

use slotmap::SlotMap;
use wgpu::{Device, MultisampleState};

use crate::gpu::render_graph::types::{ComputePipelineHandle, PipelineLayoutHandle, RenderPipelineHandle, ShaderHandle};
use std::env;
use std::path::PathBuf;

fn find_assets() -> PathBuf {
    let mut exe_path = env::current_exe().expect("Failed to get exe path");
    exe_path.pop(); // Remove the executable name, leaving the directory
    exe_path.push("assets");
    exe_path
}

 pub struct Resources {
    device: Arc<wgpu::Device>,
    assets_path: PathBuf,

    shaders: SlotMap<ShaderHandle, wgpu::ShaderModule>,

    pipelines_layouts: SlotMap<PipelineLayoutHandle, wgpu::PipelineLayout>,

    render_pipelines: SlotMap<RenderPipelineHandle, wgpu::RenderPipeline>,
    compute_pipelines: SlotMap<ComputePipelineHandle, wgpu::ComputePipeline>,
 }

 impl Resources {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Resources {
            device,
            assets_path: find_assets(),

            shaders: SlotMap::with_key(),

            pipelines_layouts: SlotMap::with_key(),

            render_pipelines: SlotMap::with_key(),
            compute_pipelines: SlotMap::with_key(),
        }
    }
    pub fn load_shader(&mut self, name: &str) -> ShaderHandle {
        let mut path = self.assets_path.clone();
        path.push("shaders");
        path.set_extension("wgsl");

        let source_code = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to find shader file at {:?}", path));

        self.shaders.insert(self.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(source_code.into())
            }
        ))
    }
    pub fn add_pipeline_layout(&mut self, desc: wgpu::PipelineLayoutDescriptor) -> PipelineLayoutHandle {
        self.pipelines_layouts.insert(
            self.device.create_pipeline_layout(&desc)
        )
    }
    pub fn create_render_pipeline(&mut self, desc: RenderPipelineDesc) -> RenderPipelineHandle {
        let descriptor = wgpu::RenderPipelineDescriptor {
            label: desc.label,
            layout: self.pipelines_layouts.get(desc.layout),
            vertex: wgpu::VertexState {
                module: self.shaders.get(desc.vertex.module).unwrap(),
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: desc.vertex.buffers,
            },
            primitive: desc.primitive,
            depth_stencil: desc.depth_stencil,
            multisample: desc.multisample,
            fragment: if let Some(fragment) = desc.fragment {
                Some(wgpu::FragmentState {
                    module: self.shaders.get(fragment.module).unwrap(),
                    entry_point: None,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: fragment.targets,
                })
            } else {None},
            multiview_mask: desc.multiview_mask,
            cache: None,
        };

        self.render_pipelines.insert(
            self.device.create_render_pipeline(&descriptor)
        )
    }
    pub fn create_compute_pipeline(&mut self, desc: wgpu::ComputePipelineDescriptor) -> ComputePipelineHandle {
        self.compute_pipelines.insert(
            self.device.create_compute_pipeline(&desc)
        )
    }
 }

 pub struct RenderPipelineDesc<'a> {
    pub label: Option<&'a str>,
    pub layout: PipelineLayoutHandle,
    pub vertex: VertexState<'a>, 
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: MultisampleState,
    pub fragment: Option<FragmentState<'a>>,
    pub multiview_mask: Option<NonZeroU32>,
 }

 pub struct VertexState<'a> {
    pub module: ShaderHandle,
    pub buffers: &'a [wgpu::VertexBufferLayout<'a>],
 }

 pub struct FragmentState<'a> {
    pub module: ShaderHandle,
    pub targets: &'a [Option<wgpu::ColorTargetState>],
 }