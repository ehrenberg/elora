//! 2D-Renderer von Elora auf Basis von wgpu (E-011).
//!
//! Formen werden zur Laufzeit per lyon tesselliert (E-033) und in einem Draw-Call
//! pro Frame gezeichnet. Weitere Pässe (z. B. egui) können sich über
//! [`Frame`] einklinken.

mod camera;
mod shapes;

use std::sync::Arc;

pub use camera::{Camera, ViewSettings};
pub use shapes::{Color, ShapeBatch};
pub use wgpu;

use shapes::Vertex;

/// Fehler beim Aufsetzen des Renderers.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Surface konnte nicht erstellt werden: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error("kein passender Grafikadapter gefunden: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),
    #[error("Grafikgerät konnte nicht erstellt werden: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("Surface wird vom Adapter nicht unterstützt")]
    UnsupportedSurface,
}

/// Ein Fenster, auf das gerendert werden kann.
pub trait RenderTarget:
    wgpu::rwh::HasWindowHandle + wgpu::rwh::HasDisplayHandle + std::fmt::Debug + Send + Sync + 'static
{
}

impl<T> RenderTarget for T where
    T: wgpu::rwh::HasWindowHandle
        + wgpu::rwh::HasDisplayHandle
        + std::fmt::Debug
        + Send
        + Sync
        + 'static
{
}

/// wgpu-Zustand plus Pipeline für farbige Formen.
#[derive(Debug)]
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    view_buffer: wgpu::Buffer,
    view_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

/// Ein laufender Frame: Ziel-Textur und Command-Encoder.
#[derive(Debug)]
pub struct Frame {
    pub encoder: wgpu::CommandEncoder,
    pub view: wgpu::TextureView,
    texture: wgpu::SurfaceTexture,
}

const INITIAL_VERTICES: u64 = 16 * 1024;

impl Renderer {
    /// Initialisiert wgpu für `window` mit der Anfangsgröße in Pixeln.
    ///
    /// # Errors
    /// Wenn kein geeigneter Adapter, kein Gerät oder keine Surface verfügbar ist.
    pub async fn new<W: RenderTarget>(
        window: Arc<W>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(window.clone())),
        );
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("elora"),
                ..Default::default()
            })
            .await?;

        let mut config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or(RenderError::UnsupportedSurface)?;
        // Nicht-sRGB-Format: Farben werden als sRGB-Werte direkt geschrieben (wie egui).
        let caps = surface.get_capabilities(&adapter);
        if let Some(format) = caps.formats.iter().find(|f| !f.is_srgb()) {
            config.format = *format;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let (pipeline, view_buffer, view_bind_group) = create_pipeline(&device, config.format);

        let vertex_buffer = create_buffer(
            &device,
            "vertices",
            INITIAL_VERTICES * size_of::<Vertex>() as u64,
            wgpu::BufferUsages::VERTEX,
        );
        let index_buffer = create_buffer(
            &device,
            "indices",
            INITIAL_VERTICES * 3 * 4,
            wgpu::BufferUsages::INDEX,
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            view_buffer,
            view_bind_group,
            vertex_buffer,
            index_buffer,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Größe der Zeichenfläche in Pixeln.
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Seitenverhältnis Breite / Höhe.
    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Beginnt einen Frame. `None`, wenn gerade nicht gezeichnet werden kann
    /// (z. B. Fenster minimiert); dann den Frame überspringen.
    pub fn begin_frame(&mut self) -> Option<Frame> {
        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return None;
            }
            _ => return None,
        };
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        Some(Frame {
            encoder,
            view,
            texture,
        })
    }

    /// Löscht den Frame mit `clear` und zeichnet alle Formen aus `batch`.
    ///
    /// # Panics
    /// Bei mehr als `u32::MAX` Indizes in einem Batch.
    pub fn draw_shapes(
        &mut self,
        frame: &mut Frame,
        camera: &Camera,
        batch: &ShapeBatch,
        clear: Color,
    ) {
        let tl = camera.top_left();
        let rect = [tl.x, tl.y, camera.size.x, camera.size.y];
        self.queue
            .write_buffer(&self.view_buffer, 0, bytemuck::cast_slice(&rect));

        let vertices = bytemuck::cast_slice(&batch.geometry.vertices);
        let indices = bytemuck::cast_slice(&batch.geometry.indices);
        ensure_capacity(
            &self.device,
            &mut self.vertex_buffer,
            "vertices",
            vertices.len(),
            wgpu::BufferUsages::VERTEX,
        );
        ensure_capacity(
            &self.device,
            &mut self.index_buffer,
            "indices",
            indices.len(),
            wgpu::BufferUsages::INDEX,
        );
        self.queue.write_buffer(&self.vertex_buffer, 0, vertices);
        self.queue.write_buffer(&self.index_buffer, 0, indices);

        let [r, g, b, a] = clear.0.map(f64::from);
        let mut pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shapes"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        if !batch.is_empty() {
            let count = u32::try_from(batch.geometry.indices.len()).expect("zu viele Indizes");
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.view_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..count, 0, 0..1);
        }
    }

    /// Schickt den Frame ab und zeigt ihn an.
    pub fn end_frame(&mut self, frame: Frame) {
        self.queue.submit([frame.encoder.finish()]);
        self.queue.present(frame.texture);
    }
}

/// Pipeline für farbige Formen samt Uniform-Buffer für den Sichtbereich.
fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::Buffer, wgpu::BindGroup) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
    let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("view"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("view"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let view_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("view"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: view_buffer.as_entire_binding(),
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("shapes"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shapes"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            })],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    (pipeline, view_buffer, view_bind_group)
}

fn create_buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// Vergrößert `buffer` bei Bedarf (Verdopplung).
fn ensure_capacity(
    device: &wgpu::Device,
    buffer: &mut wgpu::Buffer,
    label: &str,
    needed: usize,
    usage: wgpu::BufferUsages,
) {
    let needed = needed as u64;
    if needed <= buffer.size() {
        return;
    }
    let size = needed.next_power_of_two();
    *buffer = create_buffer(device, label, size, usage);
}
