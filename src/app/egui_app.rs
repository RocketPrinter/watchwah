use crate::app::SState;
use eframe::egui;
use eframe::egui::{ComboBox, ScrollArea};
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
    pub fn new(cc: &eframe::CreationContext<'_>, state: SState) -> Self {
        { state.lock().unwrap().egui_context = Some(cc.egui_ctx.clone()); }
        info!("[Client] Starting egui");
        EguiApp { state }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = self.state.lock().unwrap();

        egui::TopBottomPanel::top("bottom").show(ctx, |ui| {
            // todo: logo w context menu, ws_connected
            ui.label(format!("Connected: {0}", state.ws_connected));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Profile:");
                    ComboBox::from_label("Select profile")
                        //.selected_text(state.active_profile.as_ref().map(|p|&p.name[..]).unwrap_or("No profile selected!"))
                        .show_ui(ui, |ui| {

                        })
                });
                ui.separator();

                ui.heading("timer module");
                ui.separator();

                ui.heading("todo");
            })
        });
    }
}
