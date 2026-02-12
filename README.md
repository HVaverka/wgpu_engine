#  wGpuEngine

Rendering framework built on top of `wgpu`.


## ✨ Key Features

* **Handle-Based Resource Management**: No more fighting the borrow checker with `wgpu` lifetimes. Use `ShaderHandle`, `TextureHandle`, and `BufferHandle` to reference GPU assets stored in a centralized registry.
* **Topological Execution**: Nodes are automatically sorted based on resource dependencies. The graph ensures that any node producing a resource (e.g., a Shadow Map) executes before the nodes consuming it.
* **Persistent Resource Registry**: Shaders and Pipelines are cached and managed via a centralized `Resources` manager to prevent expensive mid-frame re-compilation.
* **Ergonomic Initialization**: Leverages `Arc<Window>` and `'static` surfaces to eliminate complex lifetime annotations in the main application loop.

---

## 🏗 Architecture

The engine is divided into three distinct layers for now:

1.  **CoreApp**: The orchestrator that manages the `winit` event loop and handles the initialization of the graphics context.
2.  **UserApp**: User defined functionality.
3.  **Resources**: The "Warehouse" where all persistent GPU objects (`wgpu::ShaderModule`, `wgpu::RenderPipeline`) live, accessible via lightweight handles.
4.  **RenderGraph**: A transient, per-frame structure where you define **Nodes** (Passes) and their resource inputs/outputs.



---

## 🛠 To-Do / Roadmap

### 1. RenderGraph Compilation
* [x] **Dependency Optimization**: Transition from $O(N^2)$ dependency checking to an $O(N)$ producer-map lookup.
* [x] **Cycle Detection**: Implement validation during the topological sort to prevent deadlocks in resource flow.

### 2. Resource Aliasing
* [x] **Lifetime Analysis**: Calculate `first_use` and `last_use` indices for all transient resources.
* [ ] **Memory Reuse**: Implement a resource pool that allows multiple transient textures to share the same memory allocation if their lifetimes do not overlap.


### 3. Compute Shader Support
* [ ] **Compute Nodes**: Add support for `ComputePipeline` execution within the RenderGraph flow.
* [ ] **Unified Barriers**: Automatic insertion of storage buffer barriers and image transitions between Compute and Graphics passes.

---

## 🚀 Quick Start

### Implement the `UserApp` Trait
    In `src/user_upp` is the App, here belongs the user code
```rust
impl UserApp for MyGame {
    fn init(resources: &mut Resources) -> Self {
        // Load shaders by name - engine handles paths internally
        let shader = resources.add_shader("main");
        
        Self { shader }
    }

    fn update(&mut self, graph: &mut RenderGraph) {
        // Define nodes and their dependencies
        graph.add_node("Main Pass")
            .execute(|encoder| {
                // Low-level wgpu commands
            });
    }
}
```