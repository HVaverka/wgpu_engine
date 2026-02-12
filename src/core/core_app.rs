use std::sync::Arc;

use wgpu::Device;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{gpu::render_graph::{graph::RenderGraph, resource_pool::Resources}, user_app};
use crate::{core::user_app::UserApp, gpu::context::WgpuCtx};
pub struct CoreApp<'window, T: UserApp> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    resources: Option<Resources>,
    user_app: Option<T>,
    exit_requested: bool,
}

impl<'window, T: UserApp> CoreApp<'window, T> {
    pub fn new() -> Self {
        Self {
            window: None,

            wgpu_ctx: None,
            resources: None,

            user_app: None,

            exit_requested: false,
        }
    }
}

impl<'window, T: UserApp> ApplicationHandler for CoreApp<'window, T> {
    // for mobile and wasm - not used, only for window creation
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = create_window(event_loop);
            let wgpu_ctx = WgpuCtx::new(Arc::clone(&window));
            let resources = Resources::new(Arc::clone(&wgpu_ctx.device));

            window.request_redraw();

            self.window = Some(window);
            self.wgpu_ctx = Some(wgpu_ctx);
            self.resources = Some(resources);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // fixed for now, need draw to get the window on screen
            WindowEvent::RedrawRequested => {
                // This is where your Render Graph logic will eventually live
                if let (Some(ctx), Some(user_app)) = (&self.wgpu_ctx, &mut self.user_app) {
                    // 1. Get the current frame from the swapchain
                    let frame = ctx
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next surface texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut render_graph = RenderGraph::new();

                    user_app.update(&mut render_graph);

                    // 2. Create a command encoder
                    let mut encoder = ctx
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    // 3. Just clear the screen to a color (this counts as "drawing")
                    {
                        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Clear Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), // Pick a visible color!
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });
                    }

                    // 4. Submit and Present
                    ctx.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
    }

    // emitted after one update
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.exit_requested {
            event_loop.exit();
        }

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn create_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let win_attr = Window::default_attributes()
        .with_title("wGPU Engine")
        .with_resizable(false)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
    // use Arc.
    Arc::new(
        event_loop
            .create_window(win_attr)
            .expect("create window err."),
    )
}
