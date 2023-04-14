mod centerer;
mod create_timer_widget;
mod duration_input_widget;
mod timer_widget;
mod top_panel;

use crate::SState;
use eframe::egui::{Context, CentralPanel, Slider, Widget};
use rodio::Source;
use rodio::source::SineWave;
use tracing::{info, instrument};
use crate::audio_manager::SoundEffects;

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
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: SState) -> Self {
        state.lock().unwrap().egui_context = Some(cc.egui_ctx.clone());

        info!("Starting egui");
        EguiApp { state }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let state = self.state.lock().unwrap();

        top_panel::ui(ctx, &state);

        CentralPanel::default().show(ctx, |ui| {
            if state.timer.is_some() {
                timer_widget::ui(ui, &state);
            } else {
                create_timer_widget::ui(ui, &state);
            }
        });
    }
}