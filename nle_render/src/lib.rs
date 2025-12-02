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
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_headless_render() {
        let engine = RenderEngine::new().await.expect("Failed to create engine");
        let device = engine.device();
        let queue = engine.queue();

        // 1. Create a texture to render to
        let texture_size = 256u32;
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm, // Use Unorm for predictable values
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Output Texture"),
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 2. Create a buffer to read pixels from
        let u32_size = std::mem::size_of::<u32>() as u32;
        let _output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
        // Pad to 256 bytes for copying
        let align = 256;
        let bytes_per_row = if (u32_size * texture_size) % align != 0 {
             ((u32_size * texture_size) / align + 1) * align
        } else {
             u32_size * texture_size
        };
        // The buffer size must be large enough to hold the padded rows
        let padded_buffer_size = (bytes_per_row * texture_size) as wgpu::BufferAddress;


        let output_buffer = device.create_buffer(&output_buffer_desc(padded_buffer_size));

        // 3. Render: Clear to Blue
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // 3b. Render Red Square (using copy for simplicity, acting as a "sprite")
        let square_size = 50u32;
        let red_data = vec![255u8, 0, 0, 255].repeat((square_size * square_size) as usize);
        let red_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: square_size,
                height: square_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            label: Some("Red Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &red_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &red_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * square_size),
                rows_per_image: Some(square_size),
            },
            wgpu::Extent3d {
                width: square_size,
                height: square_size,
                depth_or_array_layers: 1,
            },
        );

        // Copy Red Square to Output at (10, 10)
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &red_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 10, y: 10, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: square_size,
                height: square_size,
                depth_or_array_layers: 1,
            },
        );

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(texture_size),
                },
            },
            texture_desc.size,
        );

        queue.submit(Some(encoder.finish()));

        // 4. Read buffer
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        rx.await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        // Verify Pixel (10,10) is Red (255, 0, 0, 255)
        // Verify Pixel (0,0) is Blue (0, 0, 255, 255)

        // Check pixel (10,10)
        let row = 10;
        let col = 10;
        let offset = (row * bytes_per_row + col * u32_size) as usize;

        let r = data[offset];
        let g = data[offset+1];
        let b = data[offset+2];
        let a = data[offset+3];

        println!("Pixel at 10,10: {}, {}, {}, {}", r, g, b, a);

        assert_eq!(r, 255, "Red channel incorrect at 10,10");
        assert_eq!(g, 0, "Green channel incorrect at 10,10");
        assert_eq!(b, 0, "Blue channel incorrect at 10,10");
        assert_eq!(a, 255, "Alpha channel incorrect at 10,10");

        // Check pixel (0,0)
        let row = 0;
        let col = 0;
        let offset = (row * bytes_per_row + col * u32_size) as usize;

        let r = data[offset];
        let g = data[offset+1];
        let b = data[offset+2];
        let a = data[offset+3];

        println!("Pixel at 0,0: {}, {}, {}, {}", r, g, b, a);

        assert_eq!(r, 0, "Red channel incorrect at 0,0");
        assert_eq!(g, 0, "Green channel incorrect at 0,0");
        assert_eq!(b, 255, "Blue channel incorrect at 0,0");
        assert_eq!(a, 255, "Alpha channel incorrect at 0,0");

        drop(data);
        output_buffer.unmap();
    }

    fn output_buffer_desc(size: u64) -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Output Buffer"),
            mapped_at_creation: false,
        }
    }
}
