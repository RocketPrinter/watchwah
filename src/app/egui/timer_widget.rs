use chrono::Duration;
use eframe::egui::{Button, ProgressBar, Ui, Widget};
use crate::app::State;
use crate::common::timer::TimerPeriod;
use crate::common::ws_common::ClientToServer;

pub fn ui(ui: &mut Ui, state: &State) {
    let repaint_next_sec = false;

    let Some(ref timer) = state.timer else {return};

    match timer.state.period {
        TimerPeriod::Running { start, end } => {

        },
        TimerPeriod::Paused { dur_left } => {

        },
    }

    //progress_bar(ui, timer.state.elapsed, timer.state.total_dur);

    ui.vertical_centered(|ui| {
        if let TimerPeriod::Running {..} = timer.state.period {
            // todo: early stop behaviour
            if ui.add_enabled(true,Button::new("Pause")).clicked() {
                state.ws_tx.send(ClientToServer::PauseTimer).unwrap();
            }
        } else if ui.button("Unpause").clicked() {
            state.ws_tx.send(ClientToServer::UnpauseTimer).unwrap();
        }

        if ui.button("Stop").clicked() {
            state.ws_tx.send(ClientToServer::StopTimer).unwrap();
        }
    });

    ui.label(format!("{:#?}", state.timer.as_ref().unwrap()));

    if repaint_next_sec {
        if let Some(ctx) = &state.egui_context {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }
}

fn progress_bar(ui: &mut Ui, elapsed: Duration, total: Duration) {
    let secs = elapsed.num_seconds()%60;
    let mins = elapsed.num_minutes()%60;
    let hours = elapsed.num_hours();

    let progress = secs as f32 / total.num_seconds() as f32;
    let text = if hours > 0 {
        format!("{0}:{1}:{2}",hours,mins,secs)
    } else {
        format!("{0}:{1}",mins,secs)
    };

    ProgressBar::new(progress).text(text).ui(ui);
}