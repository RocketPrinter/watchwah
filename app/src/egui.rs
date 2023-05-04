mod helpers;
mod create_timer_widget;
mod timer_widget;
mod top_panel;

use crate::SState;
use eframe::egui::{Context, CentralPanel};
use tracing::{info, instrument};

#[instrument(name = "egui", skip_all)]
pub fn run(state: SState) {
    let native_options = eframe::NativeOptions {
        follow_system_theme: true,
        ..Default::default()
    };
    eframe::run_native(
        "Watchwah",
        native_options,
        Box::new(|cc| Box::new(EguiApp::new(cc, state))),
    )
    .unwrap();
}

struct EguiApp {
    state: SState,
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: SState) -> Self {
        {
            let mut state = state.lock().unwrap();
            state.egui_context = Some(cc.egui_ctx.clone());
            state.config.theme.set(cc);
        }
        info!("Starting egui");
        EguiApp { state }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let state = self.state.lock().unwrap();

        top_panel::panel(ctx, &state);

        CentralPanel::default().show(ctx, |ui| {
            if state.timer.is_some() {
                timer_widget::ui(ui, &state);
            } else {
                create_timer_widget::ui(ui, &state);
            }
        });
    }
}