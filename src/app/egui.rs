mod create_timer_widget;
mod duration_widget;
mod timer_widget;

use crate::app::SState;
use eframe::egui;
use eframe::egui::{Button, ComboBox, ScrollArea};
use tracing::{info, instrument};

#[instrument(name = "egui", skip_all)]
pub fn run(state: SState) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Watchwah",
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc, state))),
    )
    .unwrap();
}

struct EguiApp {
    state: SState,

    owo: i32,
    create_timer: create_timer_widget::CreateTimerState,
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: SState) -> Self {
        {
            state.lock().unwrap().egui_context = Some(cc.egui_ctx.clone());
        }
        info!("Starting egui");
        EguiApp {
            state,

            owo: 0,
            create_timer: create_timer_widget::CreateTimerState::default(),
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();

        egui::TopBottomPanel::top("top")
            .min_height(0.)
            .show(ctx, |ui| {
                // todo: logo w context menu, ws_connected
                ui.label(format!("Connected: {0}", state.ws_connected));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.owo.to_string());
            self.owo += 1;

            if state.timer.is_some() {
                timer_widget::ui(ui, &state);
            } else {
                create_timer_widget::ui(ui, &mut self.create_timer, &state);
            }
        });
    }
}
