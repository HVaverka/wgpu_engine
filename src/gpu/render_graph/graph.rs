use std::collections::{HashMap, VecDeque};

use bytemuck::{Pod, Zeroable};
use wgpu::{CommandEncoder, RenderPassDescriptor, wgt::CommandEncoderDescriptor};

use crate::gpu::render_graph::registry::InstanceRegistry;
use crate::gpu::render_graph::types::{
    BufferDesc, BufferHandle, CopyOp, DownloadOp, Node, NodeInput, NodeOutput, NodeType,
    PipelineHandle, ResourceHandle, TextureDesc, TextureHandle, UploadOp,
};

pub struct RenderGraph {
    nodes: Vec<Node>,

    textures: InstanceRegistry<TextureHandle, TextureDesc>,
    buffers: InstanceRegistry<BufferHandle, BufferDesc>,

    writers: HashMap<ResourceHandle, Vec<NodeHandle>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        RenderGraph {
            nodes: Vec::new(),

            textures: InstanceRegistry::new(),
            buffers: InstanceRegistry::new(),

            writers: HashMap::new(),
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
        PassBuilder {
            graph: self,
            name: name.into(),
            kind: kind,
            inputs: Vec::new(),
            outputs: Vec::new(),
            next_bind_idx: 0,
            pipeline: None,
        }
    }

    pub fn compile(&self, device: &mut wgpu::Device) {
        let order = self.get_node_order();
        if let Err(()) = order {
            return;
        }

        let order = order.unwrap();

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Command encoder"),
        });

        for &idx in order.iter() {
            let node = &self.nodes[idx];

            match node.kind {
                NodeType::RenderPass => {
                    self.compile_render_pass(node, device, &mut encoder);
                }
                NodeType::ComputePass => {}
                NodeType::Transfer => {}
            }
        }
    }

    fn compile_tranfer() {}

    fn compile_render_pass(
        &self,
        node: &Node,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        //render_pass.set_pipeline(self.pipelines.);
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
    next_bind_idx: u32,

    pipeline: Option<PipelineHandle>,
}

impl<'a> PassBuilder<'a> {
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

    pub fn use_pipeline(mut self, pipeline: PipelineHandle) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn execute<F>(mut self, func: F)
    where
        F: FnOnce(&mut CommandEncoder) + 'static,
    {
        let pass = Node {
            name: self.name,
            kind: self.kind,
            inputs: self.inputs,
            outputs: self.outputs,
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
