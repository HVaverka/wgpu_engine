use wgpu::CommandEncoder;

use crate::gpu::render_graph::types::{Pass, PassInput, PassOutput, PassType, PipelineDesc, PipelineHandle, ResourceDesc, ResourceHandle};

pub struct RenderGraph {
    passes: Vec<Pass>,
    resources: Vec<ResourceDesc>,
    pipelines: Vec<PipelineDesc>,
}

impl RenderGraph {
    pub fn new() -> Self {
        RenderGraph {
            passes: Vec::new(),
            resources: Vec::new(),
            pipelines: Vec::new(),
        }
    }

    pub fn add_resource(&mut self, desc: ResourceDesc) -> ResourceHandle {
        let handle = ResourceHandle(self.resources.len() as u32);
        self.resources.push(desc);

        handle
    }

    pub fn add_pipeline(&mut self, desc: PipelineDesc) -> PipelineHandle {
        let handle = PipelineHandle(self.pipelines.len() as u32);
        self.pipelines.push(desc);
        handle
    }

    pub fn add_pass(&mut self, name: &str, kind: PassType) -> PassBuilder {
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
}

pub struct PassBuilder<'a> {
    graph: &'a mut RenderGraph,
    name: String,
    kind: PassType,

    inputs: Vec<PassInput>,
    outputs: Vec<PassOutput>,
    next_bind_idx: u32,

    pipeline: Option<PipelineHandle>,
}

impl<'a> PassBuilder<'a> {
    pub fn read(mut self, resource: ResourceHandle) -> Self {
        let binding =  self.get_next_bind_idx();
        self.inputs.push(PassInput{binding, resource});
        self
    }

    pub fn write(mut self, resource: ResourceHandle) -> Self {
        let binding = self.get_next_bind_idx();
        self.outputs.push(PassOutput{binding, resource});
        self
    }

    pub fn read_write(mut self, resource: ResourceHandle) -> Self {
        let binding = self.get_next_bind_idx();
        self.inputs.push(PassInput { binding, resource });
        self.outputs.push(PassOutput { binding, resource });
        self
    }

    pub fn use_pipeline(mut self, pipeline: PipelineHandle) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn execute<F>(mut self, func: F) 
    where F: FnOnce(&mut CommandEncoder) + 'static
    {
        let pass = Pass {
            name: self.name,
            kind: self.kind,
            inputs: self.inputs,
            outputs: self.outputs,
            pipeline: self.pipeline,
            execute: Box::new(func),
        };
        self.graph.passes.push(pass);
    }

    fn get_next_bind_idx(&mut self) -> u32 {
        let binding = self.next_bind_idx;
        self.next_bind_idx += 1;
        binding
    }
}