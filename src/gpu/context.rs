use std::sync::Arc;

use wgpu::{ExperimentalFeatures, Features, wgc::{device, instance}, wgt::DeviceDescriptor};
use winit::window::{self, Window};

pub struct WgpuCtx<'window> {
    pub surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,

    adapter: wgpu::Adapter,
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
}

impl<'window> WgpuCtx<'window> {
    pub fn new(window: Arc<Window>) -> Self {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    async fn new_async(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find appropriate adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device"),
                required_features: Features::empty(),
                required_limits: adapter.limits(),
                experimental_features: unsafe { ExperimentalFeatures::enabled() },
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to aquire device");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let mut surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface_config.format = wgpu::TextureFormat::Rgba8UnormSrgb;
        surface_config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &surface_config);

        let device = Arc::new(device);

        Self {
            surface,
            surface_config,
            adapter,
            device,
            queue,
        }
    }
}
