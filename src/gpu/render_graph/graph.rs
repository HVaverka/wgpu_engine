use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use slotmap::SparseSecondaryMap;
use wgpu::{CommandEncoder, RenderPassDescriptor, wgt::CommandEncoderDescriptor};

use crate::gpu::render_graph::registry::InstanceRegistry;
use crate::gpu::render_graph::resource_pool::Resources;
use crate::gpu::render_graph::types::{
    BufferDesc, BufferHandle, CopyOp, DownloadOp, Node, NodeInput, NodeOutput, NodeType, PassContext, PipelineHandle, ResourceHandle, ResourceType, TextureDesc, TextureHandle, UploadOp
};

pub struct RenderGraph {
    nodes: Vec<Node>,

    textures: InstanceRegistry<TextureHandle, TextureDesc>,
    buffers: InstanceRegistry<BufferHandle, BufferDesc>,

    texture_cache: HashMap<TextureDesc, CachedTexture>,
    buffer_cache: HashMap<BufferDesc, CachedBuffer>,
}

impl RenderGraph {
    pub fn new() -> Self {
        RenderGraph {
            nodes: Vec::new(),

            textures: InstanceRegistry::new(),
            buffers: InstanceRegistry::new(),

            texture_cache: HashMap::new(),
            buffer_cache: HashMap::new(),
        }
    }

    pub fn add_texture(&mut self, desc: TextureDesc) -> ResourceHandle {
        ResourceHandle::Texture(self.textures.insert(desc))
    }

    pub fn add_buffer(&mut self, desc: BufferDesc) -> ResourceHandle {
        ResourceHandle::Buffer(self.buffers.insert(desc))
    }

    pub fn add_transfer(&mut self, name: &str) -> TransferBuilder {
        TransferBuilder {
            graph: self,
            name: name.into(),
            kind: NodeType::Transfer,

            upload_op: Vec::new(),
            download_op: Vec::new(),
            copy_op: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, name: &str, kind: NodeType) -> PassBuilder {
        PassBuilder::new(self, name, kind)
    }

    pub fn compile(&mut self, device: &wgpu::Device, resources: &Resources) {
        let order = self.get_node_order();
        if let Err(()) = order {
            return;
        }

        let order = order.unwrap();

        let mut texture_lt: SparseSecondaryMap<TextureHandle, ResourceLifetime> = SparseSecondaryMap::new();
        let mut buffer_lt: SparseSecondaryMap<BufferHandle, ResourceLifetime> = SparseSecondaryMap::new();

        for &i in order.iter() {
            let node = &self.nodes[i];

            let mut process_resource = |res: &ResourceHandle| {
                match res {
                    ResourceHandle::Buffer(handle) => {
                        if let Some(lt) = buffer_lt.get_mut(*handle) {
                            lt.last_use = i as u32;
                        } else {
                            buffer_lt.insert(*handle, ResourceLifetime { first_use: i as u32, last_use: i as u32 });
                        }
                    },
                    ResourceHandle::Texture(handle) => {
                        if let Some(lt) = texture_lt.get_mut(*handle) {
                            lt.last_use = i as u32;
                        } else {
                            texture_lt.insert(*handle, ResourceLifetime { first_use: i as u32, last_use: i as u32 });
                        }
                    }
                }
            };

            for input in &node.inputs { process_resource(&input.resource); }
            for output in &node.outputs { process_resource(&output.resource); }
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command encoder"),
        });

        for &idx in order.iter() {
            let node = &self.nodes[idx];

            match node.kind {
                NodeType::RenderPass => {
                    self.compile_render_pass(idx, device, &mut encoder);
                }
                NodeType::ComputePass => {}
                NodeType::Transfer => {}
            }
        }
    }

    fn compile_tranfer() {}

    fn compile_render_pass(
        &mut self,
        node_idx: usize,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let node = &mut self.nodes[node_idx];
        let mut frame_views = Vec::new();
        let mut color_attachments = Vec::new();

        for output in node.outputs.iter() {
            let ResourceHandle::Texture(handle) = output.resource else {continue;};

            let desc = self.textures.get(handle).expect("Texture does not exist");
            
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&output.binding.to_string()),
                size: desc.size,
                mip_level_count: 0,
                sample_count: 0,
                dimension: desc.dimension,
                format: desc.format,
                usage: desc.usage,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            frame_views.push(view);

            self.texture_cache.insert(*desc, CachedTexture {
                texture: texture,
            });
        }

        for view in frame_views.iter() {
            color_attachments.push(Some(wgpu::RenderPassColorAttachment {
                view: view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }));
        }

        let depth_data = node.depth_texture.map(|handle| {
            let ResourceHandle::Texture(h) = handle else { panic!("Depth must be a texture") };
            let desc = self.textures.get(h).expect("Depth texture desc missing");
            
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth texture"),
                size: desc.size,
                mip_level_count: 0,
                sample_count: 0,
                dimension: desc.dimension,
                format: desc.format,
                usage: desc.usage,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            (texture, view)
        });
        // todo safe the depth texture for reuse

        let depth_stencil_attachment = depth_data.as_ref().map(|(_tex, view)| {
            wgpu::RenderPassDepthStencilAttachment {
                view, // This is a &TextureView
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }
        });

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some(&node.name),
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };


        let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);

        
        if let Some(ctx) = node.execute.take() {
            ctx(PassContext::Render(&mut render_pass));
        }
    }

    fn get_node_order(&self) -> Result<Vec<usize>, ()> {
        let mut writers = HashMap::new();

        for (idx, node) in self.nodes.iter().enumerate() {
            for res in node.outputs.iter() {
                writers.insert(res.resource, idx);
            }
        }

        let node_count = self.nodes.len();

        let mut edges: Vec<Vec<usize>> = vec![Vec::new(); node_count];
        let mut dependency_count = vec![0; node_count];

        for (reader_idx, node) in self.nodes.iter().enumerate() {
            for input in node.inputs.iter() {
                if let Some(&writer_idx) = writers.get(&input.resource) {
                    edges[writer_idx].push(reader_idx);
                    dependency_count[reader_idx] += 1;
                }
            }
        }

        let mut queue = VecDeque::new();
        let mut order = Vec::new();

        for i in 0..node_count {
            if dependency_count[i] == 0 {
                queue.push_back(i);
            }
        }

        while let Some(u) = queue.pop_front() {
            order.push(u);

            for &v in &edges[u] {
                dependency_count[v] -= 1;
                if dependency_count[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        if order.len() != node_count {
            return Err(());
        }

        return Ok(order);
    }
}

pub struct TransferBuilder<'a> {
    graph: &'a mut RenderGraph,
    name: String,
    kind: NodeType,

    upload_op: Vec<UploadOp>,
    download_op: Vec<DownloadOp>,
    copy_op: Vec<CopyOp>,
    // readback_tickets: Vec<ReadbackTicket<T>>,
}

impl<'a> TransferBuilder<'a> {
    pub fn read<T: Pod + Zeroable>(
        mut self,
        source: ResourceHandle,
        offset: u64,
        size: u64,
    ) -> Self {
        !todo!();
        self
    }

    pub fn write<T: Pod + Zeroable>(
        mut self,
        dest: ResourceHandle,
        offset: u64,
        data: Vec<T>,
    ) -> Self {
        self.upload_op.push(UploadOp {
            target: dest,
            offset: offset,
            data: bytemuck::cast_vec(data),
        });
        self
    }

    pub fn copy(
        mut self,
        src: ResourceHandle,
        dst: ResourceHandle,
        size: u64,
        src_offset: u64,
        dst_offset: u64,
    ) -> Self {
        self.copy_op.push(CopyOp {
            src,
            dst,
            size,
            src_offset,
            dst_offset,
        });
        self
    }

    pub fn finish(mut self) {
        let transfer = Node {
            name: self.name,
            kind: self.kind,
            inputs: Vec::new(),
            outputs: Vec::new(),
            depth_texture: None,
            pipeline: None,
            execute: None,
        };

        self.graph.nodes.push(transfer);
    }
}

pub struct PassBuilder<'a> {
    graph: &'a mut RenderGraph,
    name: String,
    kind: NodeType,

    inputs: Vec<NodeInput>,
    outputs: Vec<NodeOutput>,
    depth_texture: Option<ResourceHandle>,
    next_bind_idx: u32,

    pipeline: Option<PipelineHandle>,
}

impl<'a> PassBuilder<'a> {
    pub fn new(graph: &'a mut RenderGraph, name: &str, kind: NodeType) -> Self {
        PassBuilder {
            graph: graph,
            name: name.into(),
            kind: kind,
            inputs: Vec::new(),
            outputs: Vec::new(),
            depth_texture: None,
            next_bind_idx: 0,
            pipeline: None,
        }
    }

    pub fn read(mut self, resource: ResourceHandle) -> Self {
        let binding = self.get_next_bind_idx();
        self.inputs.push(NodeInput { binding, resource });
        self
    }

    pub fn write(mut self, resource: ResourceHandle) -> Self {
        let binding = self.get_next_bind_idx();
        self.outputs.push(NodeOutput { binding, resource });
        self
    }

    pub fn read_write(mut self, resource: ResourceHandle) -> Self {
        let binding = self.get_next_bind_idx();
        self.inputs.push(NodeInput { binding, resource });
        self.outputs.push(NodeOutput { binding, resource });
        self
    }

    pub fn write_depth(mut self, resource: ResourceHandle) -> Self {
        self.depth_texture = Some(resource);
        self
    }

    pub fn use_pipeline(mut self, pipeline: PipelineHandle) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn execute<F>(mut self, func: F)
    where
        F: FnOnce(PassContext) + 'static,
    {
        let pass = Node {
            name: self.name,
            kind: self.kind,
            inputs: self.inputs,
            outputs: self.outputs,
            depth_texture: self.depth_texture,
            pipeline: self.pipeline,
            execute: Some(Box::new(func)),
        };
        self.graph.nodes.push(pass);
    }

    fn get_next_bind_idx(&mut self) -> u32 {
        let binding = self.next_bind_idx;
        self.next_bind_idx += 1;
        binding
    }
}

#[derive(Clone, Copy)]
struct NodeHandle {
    idx: usize,
}

struct ResourceLifetime {
    first_use: u32,
    last_use: u32,
}

struct CachedTexture {
    texture: wgpu::Texture,
}
struct CachedBuffer {
    buffer: wgpu::Buffer,
}

