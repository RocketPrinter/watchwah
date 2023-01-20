use eframe::egui;
use tracing::info;
use crate::app::SState;

pub struct EguiApp {
    state: SState,
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: SState) -> Self {
        info!("[Client] Starting egui");


        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        EguiApp {
            state,
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}