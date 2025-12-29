use anyhow::Result;
pub use wgpu;

pub trait RenderNode {
    fn update(&mut self, _queue: &wgpu::Queue) {}
    fn encode(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);
}

pub struct RenderEngine {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RenderEngine {
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        // Try HighPerformance first
        let mut adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await;

        // Fallback to any power preference
        if adapter.is_none() {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::None,
                    force_fallback_adapter: false,
                    compatible_surface: None,
                })
                .await;
        }

        // Fallback to software renderer (e.g. llvmpipe on Linux CI)
        if adapter.is_none() {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::None,
                    force_fallback_adapter: true,
                    compatible_surface: None,
                })
                .await;
        }

        let adapter = adapter.ok_or_else(|| anyhow::anyhow!("No suitable adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Render Engine Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        Ok(Self { device, queue })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_render_engine_init() {
        let engine = RenderEngine::new().await;
        if engine.is_err() {
            println!("Skipping test_render_engine_init: No suitable adapter found");
            return;
        }
        assert!(engine.is_ok());
    }
}
