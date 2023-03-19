use chrono::Duration;
use eframe::egui::{Button, Color32, ProgressBar, Ui, vec2, Widget};
use crate::app::egui::centerer::centerer;
use crate::app::State;
use crate::common::timer::{PomodoroPeriod, PomodoroState, Timer, TimerPeriod, TimerState};
use crate::common::ws_common::ClientToServer;

pub fn ui(ui: &mut Ui, state: &State) {
    let Some(ref timer) = state.timer else {return};

    let (color,title) = color_and_title(&timer.state);

    ui.vertical_centered(|ui| {
        // title
        ui.heading(title);





        buttons(ui, state);
    });




    ///progress_bar(ui, timer.state.elapsed, timer.state.total_dur);

    ui.label(format!("{:#?}", state.timer.as_ref().unwrap()));
}

fn color_and_title(timer_state: &TimerState) -> (Color32, &'static str) {
    if let TimerPeriod::Paused {..} = timer_state.period {
        // paused
        return (Color32::from_rgb(255, 255, 255), "Paused")
    }
    match timer_state.pomodoro.as_ref().map(|p| &p.current_period) {
        Some(PomodoroPeriod::ShortBreak) => (Color32::from_rgb(255, 255, 255), "Break"),
        Some(PomodoroPeriod::LongBreak) => (Color32::from_rgb(255, 255, 255), "Long Break"),
        Some(PomodoroPeriod::Work) | None => (Color32::from_rgb(255, 255, 255), "Work"),
    }
}

fn time_progress_bar(ui: &mut Ui, color: Color32, elapsed: Duration, total: Duration) {
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

fn goal_progress_bar(ui: &mut Ui, timer: &Timer) {
    todo!()
}

fn buttons(ui: &mut Ui, state: &State) {
    centerer(ui, |ui| {
        if let TimerPeriod::Running {..} = state.timer.as_ref().unwrap().state.period {
            // todo: early stop behaviour

            if ui.add_enabled(true,Button::new("Pause").min_size(vec2(70.,1.))).clicked() {
                state.ws_tx.send(ClientToServer::PauseTimer).unwrap();
            }
        } else if ui.add(Button::new("Unpause").min_size(vec2(70.,1.))).clicked() {
            state.ws_tx.send(ClientToServer::UnpauseTimer).unwrap();
        }

        if ui.add(Button::new("Stop").min_size(vec2(70.,1.))).clicked() {
            state.ws_tx.send(ClientToServer::StopTimer).unwrap();
        }
    });
}