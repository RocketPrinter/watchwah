use crate::app::SState;
use eframe::egui;
use tracing::info;

pub fn run(state: SState) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Watchwah",
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc, state))),
    );
}

struct EguiApp {
    state: SState,
}

impl EguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: SState) -> Self {
        info!("[Client] Startzing egui");

        EguiApp { state }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
