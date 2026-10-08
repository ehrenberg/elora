//! A full-screen picture that is replaced every frame: the adventure intro video (I-2, E-355).
//! The picture is fitted into the screen with its aspect ratio; around it the frame is black.

use crate::create_buffer;

/// Texture, pipeline and fitting of the current picture.
#[derive(Debug)]
pub(crate) struct Picture {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    /// Texture of the current size plus bind group; recreated when the size changes.
    texture: Option<(wgpu::Texture, wgpu::BindGroup, u32, u32)>,
}

impl Picture {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("picture"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("picture"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipeline: create_pipeline(device, format, samples, &layout),
            layout,
            sampler: linear,
            uniform: create_buffer(device, "picture", 16, wgpu::BufferUsages::UNIFORM),
            texture: None,
        }
    }

    /// Uploads `rgba` (`width × height` pixels, row after row).
    pub(crate) fn set(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) {
        if !matches!(self.texture, Some((_, _, w, h)) if w == width && h == height) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("picture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // non-sRGB like the surface: the bytes go to the screen unchanged
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("picture"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform.as_entire_binding(),
                    },
                ],
            });
            self.texture = Some((texture, bind_group, width, height));
        }
        let Some((texture, ..)) = &self.texture else {
            return;
        };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub(crate) fn clear(&mut self) {
        self.texture = None;
    }

    /// Writes the fitting for a screen of `screen` pixels; `None` without a picture.
    pub(crate) fn prepare(
        &self,
        queue: &wgpu::Queue,
        screen: (u32, u32),
    ) -> Option<&wgpu::BindGroup> {
        let (_, bind_group, w, h) = self.texture.as_ref()?;
        let scale = fit(*w, *h, screen.0, screen.1);
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::cast_slice(&[scale[0], scale[1], 0.0, 0.0]),
        );
        Some(bind_group)
    }
}

/// Size of a `w × h` picture fitted into a `sw × sh` screen, in clip units (1 = full).
pub(crate) fn fit(w: u32, h: u32, sw: u32, sh: u32) -> [f32; 2] {
    let picture = w as f32 / h.max(1) as f32;
    let screen = sw as f32 / sh.max(1) as f32;
    if picture > screen {
        [1.0, screen / picture]
    } else {
        [picture / screen, 1.0]
    }
}

pub(crate) fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    samples: u32,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("picture.wgsl"));
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("picture"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("picture"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
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
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn letterbox_and_pillarbox() {
        let [x, y] = fit(1280, 720, 1280, 720);
        assert!((x - 1.0).abs() < 1e-6 && (y - 1.0).abs() < 1e-6);
        // 16:9 video on a 4:3 screen: bars above and below
        let [x, y] = fit(1280, 720, 1024, 768);
        assert!((x - 1.0).abs() < 1e-6 && (y - 0.75).abs() < 1e-6);
        // on an ultra-wide screen: bars left and right
        let [x, y] = fit(1280, 720, 2560, 1080);
        assert!((y - 1.0).abs() < 1e-6 && x < 0.9);
    }
}
