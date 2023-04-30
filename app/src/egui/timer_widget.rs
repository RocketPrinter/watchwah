use chrono::Duration;
use eframe::egui::{Button, Color32, ProgressBar, RichText, Ui, vec2, Widget};
use core::time::Duration as StdDuration;
use crate::egui::centerer::centerer;
use crate::State;
use common::timer::{PeriodProgress, PeriodType, Timer, TimerState};
use common::ws_common::ClientToServer;

// todo: incomplete, rewrite whole thing
pub fn ui(ui: &mut Ui, state: &State) {
    let Some(ref timer) = state.timer else {return};

    let (_color,title) = color_and_title(&timer.state);

    ui.vertical_centered(|ui| {
        // title
        ui.heading(title);

        time_progress_bar(ui, &timer.state.progress);

        buttons(ui, state, timer);
    });
}

fn color_and_title(timer_state: &TimerState) -> (Color32, &'static str) {
    if let PeriodProgress::Paused {..} = timer_state.progress {
        return (Color32::from_rgb(255, 255, 255), "Paused")
    }

    match &timer_state.period {
        PeriodType::Uninit => (Color32::from_rgb(255, 255, 255), "Uninit"),
        PeriodType::Work => (Color32::from_rgb(255, 255, 255), "Work"),
        PeriodType::StartingBreak => (Color32::from_rgb(255, 255, 255), "Starting Break"),
        PeriodType::ShortBreak => (Color32::from_rgb(255, 255, 255), "Break"),
        PeriodType::LongBreak  => (Color32::from_rgb(255, 255, 255), "Long Break"),
    }
}

fn time_progress_bar(ui: &mut Ui, period: &PeriodProgress) {
    let elapsed = period.elapsed();

    if let Some(limit) = period.limit() {
        let progress = elapsed.num_milliseconds() as f32 / limit.num_milliseconds() as f32;
        let diff_text = format_dur(limit - elapsed);

        // if the time limit is low we update every frame to make the bar smoother
        if limit < Duration::minutes(5) {
            ui.ctx().request_repaint();
        } else {
            ui.ctx().request_repaint_after(StdDuration::from_millis(200));
        }

        ProgressBar::new(progress).text(diff_text).ui(ui);
    } else {
        ui.ctx().request_repaint_after(StdDuration::from_millis(200));

        ui.label(RichText::new(format_dur(elapsed)).size(30.));
    }

    fn format_dur(dur: Duration) -> String {
        let secs = dur.num_seconds()%60;
        let mins = dur.num_minutes()%60;
        let hours = dur.num_hours();

        if hours > 0 {
            format!("{0}:{1:0>2}:{2:0>2}",hours,mins,secs)
        } else {
            format!("{0:0>2}:{1:0>2}",mins,secs)
        }
    }
}

fn buttons(ui: &mut Ui, state: &State, timer: &Timer) {
    centerer(ui, |ui| {
        if let PeriodProgress::Running {..} = timer.state.progress {
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

        // skipping only makes sense if pomodoro is enabled
        if timer.profile.pomodoro.is_some() {
            let enabled = timer.profile.can_skip_work || !matches!(timer.state.period, PeriodType::Work);
            if ui.add_enabled(enabled, Button::new("Skip").min_size(vec2(70.,1.))).clicked() {
                state.ws_tx.send(ClientToServer::SkipPeriod).unwrap();
            }
        }
    });
}