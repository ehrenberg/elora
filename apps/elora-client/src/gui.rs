//! egui binding (E-031): input via egui-winit, drawing via egui-wgpu.

use elora_render::{Frame, Renderer, wgpu};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct Gui {
    pub ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

/// Custom fonts instead of the egui default fonts (E-047): Inter and `JetBrains Mono` (OFL-1.1).
fn fonts() -> egui::FontDefinitions {
    use std::sync::Arc;

    let mut fonts = egui::FontDefinitions::empty();
    fonts.font_data.insert(
        "Inter".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "JetBrainsMono".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec!["Inter".into(), "JetBrainsMono".into()],
    );
    fonts.families.insert(
        egui::FontFamily::Monospace,
        vec!["JetBrainsMono".into(), "Inter".into()],
    );
    fonts
}

impl std::fmt::Debug for Gui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gui").finish_non_exhaustive()
    }
}

impl Gui {
    pub fn new(window: &Window, renderer: &Renderer) -> Self {
        let ctx = egui::Context::default();
        ctx.set_fonts(fonts());
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(renderer.device().limits().max_texture_dimension_2d as usize),
        );
        let renderer = egui_wgpu::Renderer::new(
            renderer.device(),
            renderer.format(),
            egui_wgpu::RendererOptions::default(),
        );
        Self {
            ctx,
            state,
            renderer,
        }
    }

    /// Returns `true` if egui consumed the event.
    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn wants_pointer(&self) -> bool {
        self.ctx.egui_wants_pointer_input()
    }

    /// Builds the UI with `ui` and draws it over the existing frame content.
    pub fn draw(
        &mut self,
        window: &Window,
        gfx: &Renderer,
        frame: &mut Frame,
        ui: impl FnMut(&mut egui::Ui),
    ) {
        let input = self.state.take_egui_input(window);
        let mut output = self.ctx.run_ui(input, ui);
        self.state
            .handle_platform_output(window, output.platform_output);

        let ppp = output.pixels_per_point;
        let jobs = self.ctx.tessellate(output.shapes, ppp);
        let (w, h) = gfx.size();
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [w, h],
            pixels_per_point: ppp,
        };

        let mut textures = std::mem::take(&mut output.textures_delta);
        for (id, deltas) in textures.set.drain() {
            for delta in &deltas {
                self.renderer
                    .update_texture(gfx.device(), gfx.queue(), id, delta);
            }
        }
        let extra = self.renderer.update_buffers(
            gfx.device(),
            gfx.queue(),
            &mut frame.encoder,
            &jobs,
            &screen,
        );
        gfx.queue().submit(extra);
        {
            let pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            self.renderer
                .render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        for id in textures.free.drain() {
            self.renderer.free_texture(&id);
        }
    }
}
