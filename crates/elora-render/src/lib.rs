//! 2D-Renderer von Elora auf Basis von wgpu (E-011).
//!
//! Formen werden zur Laufzeit per lyon tesselliert (E-033) und in einem Draw-Call
//! pro Frame gezeichnet. Wiederkehrende Formen (Figur, Pickups) liegen als
//! gecachte [`Mesh`]es vor und werden nur noch transformiert (M5.1). Kanten werden
//! per MSAA geglättet. Weitere Pässe (z. B. egui) können sich über
//! [`Frame`] einklinken.

mod camera;
mod mesh;
mod shapes;
mod svg;
mod text;

use std::sync::Arc;

pub use camera::{Camera, ViewSettings};
/// Pfade für [`MeshBuilder`].
pub use lyon::path::Path;
pub use mesh::{Affine, Mesh, MeshBuilder, Paint, Tint, ellipse, lerp_color, rounded_rect, shade};
pub use shapes::{Color, ShapeBatch};
pub use svg::{SvgAsset, SvgError};
pub use text::{Align, Font, FontError};
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
    /// Welt (Weltkoordinaten) und Overlay (z. B. HUD in Bildschirmkoordinaten)
    /// brauchen eigene Puffer, weil beide im selben Frame beschrieben werden.
    world: Layer,
    overlay: Layer,
    /// Abtastungen je Pixel (1 = kein MSAA).
    samples: u32,
    /// Mehrfach abgetastetes Ziel, wird in die Surface aufgelöst.
    msaa: Option<wgpu::TextureView>,
}

/// Ein laufender Frame: Ziel-Textur und Command-Encoder.
#[derive(Debug)]
pub struct Frame {
    pub encoder: wgpu::CommandEncoder,
    pub view: wgpu::TextureView,
    texture: wgpu::SurfaceTexture,
}

const INITIAL_VERTICES: u64 = 16 * 1024;

/// Gewünschte MSAA-Stufe; fällt auf 1 zurück, wenn das Format sie nicht kann.
const MSAA_SAMPLES: u32 = 4;

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

        let samples = if adapter
            .get_texture_format_features(config.format)
            .flags
            .sample_count_supported(MSAA_SAMPLES)
        {
            MSAA_SAMPLES
        } else {
            1
        };
        let (pipeline, layout) = create_pipeline(&device, config.format, samples);
        let world = Layer::new(&device, &layout, "world");
        let overlay = Layer::new(&device, &layout, "overlay");
        let msaa = create_msaa(&device, &config, samples);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            world,
            overlay,
            samples,
            msaa,
        })
    }

    /// MSAA-Stufe (1 = aus).
    pub fn msaa_samples(&self) -> u32 {
        self.samples
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
        self.msaa = create_msaa(&self.device, &self.config, self.samples);
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
        self.draw_layer(frame, false, camera, batch, Some(clear));
    }

    /// Zeichnet `batch` über den bisherigen Frame (nach [`Renderer::draw_shapes`]),
    /// z. B. das HUD mit einer Kamera in Bildschirm-Pixeln.
    ///
    /// # Panics
    /// Bei mehr als `u32::MAX` Indizes in einem Batch.
    pub fn draw_overlay(&mut self, frame: &mut Frame, camera: &Camera, batch: &ShapeBatch) {
        self.draw_layer(frame, true, camera, batch, None);
    }

    fn draw_layer(
        &mut self,
        frame: &mut Frame,
        overlay: bool,
        camera: &Camera,
        batch: &ShapeBatch,
        clear: Option<Color>,
    ) {
        let layer = if overlay {
            &mut self.overlay
        } else {
            &mut self.world
        };
        layer.upload(&self.device, &self.queue, camera, batch);
        let load = match clear {
            Some(color) => {
                let [r, g, b, a] = color.0.map(f64::from);
                wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a })
            }
            None => wgpu::LoadOp::Load,
        };
        let mut pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(if overlay { "overlay" } else { "shapes" }),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.msaa.as_ref().unwrap_or(&frame.view),
                    depth_slice: None,
                    resolve_target: self.msaa.as_ref().map(|_| &frame.view),
                    ops: wgpu::Operations {
                        load,
                        // MSAA-Inhalt behalten, damit das Overlay darauf aufsetzen kann
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
            pass.set_bind_group(0, &layer.bind_group, &[]);
            pass.set_vertex_buffer(0, layer.vertices.slice(..));
            pass.set_index_buffer(layer.indices.slice(..), wgpu::IndexFormat::Uint32);
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
    samples: u32,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
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
        multisample: wgpu::MultisampleState {
            count: samples,
            ..Default::default()
        },
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
    (pipeline, layout)
}

/// Puffer für einen Zeichendurchgang: Sichtbereich, Vertices, Indizes.
#[derive(Debug)]
struct Layer {
    view: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
}

impl Layer {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, label: &str) -> Self {
        let view = create_buffer(device, label, 16, wgpu::BufferUsages::UNIFORM);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view.as_entire_binding(),
            }],
        });
        Self {
            view,
            bind_group,
            vertices: create_buffer(
                device,
                label,
                INITIAL_VERTICES * size_of::<Vertex>() as u64,
                wgpu::BufferUsages::VERTEX,
            ),
            indices: create_buffer(
                device,
                label,
                INITIAL_VERTICES * 3 * 4,
                wgpu::BufferUsages::INDEX,
            ),
        }
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera,
        batch: &ShapeBatch,
    ) {
        let tl = camera.top_left();
        let rect = [tl.x, tl.y, camera.size.x, camera.size.y];
        queue.write_buffer(&self.view, 0, bytemuck::cast_slice(&rect));
        let vertices = bytemuck::cast_slice(&batch.geometry.vertices);
        let indices = bytemuck::cast_slice(&batch.geometry.indices);
        ensure_capacity(
            device,
            &mut self.vertices,
            "vertices",
            vertices.len(),
            wgpu::BufferUsages::VERTEX,
        );
        ensure_capacity(
            device,
            &mut self.indices,
            "indices",
            indices.len(),
            wgpu::BufferUsages::INDEX,
        );
        queue.write_buffer(&self.vertices, 0, vertices);
        queue.write_buffer(&self.indices, 0, indices);
    }
}

/// MSAA-Ziel in Surface-Größe; `None` ohne MSAA.
fn create_msaa(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    samples: u32,
) -> Option<wgpu::TextureView> {
    (samples > 1).then(|| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    })
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
