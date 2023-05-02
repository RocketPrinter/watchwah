use chrono::Duration;
use eframe::egui::{Button, Color32, popup_below_widget, ProgressBar, Response, RichText, Ui, vec2, Widget};
use core::time::Duration as StdDuration;
use std::hash::Hash;
use crate::egui::centerer::centerer;
use crate::State;
use common::timer::{PeriodProgress, PeriodType, Timer, TimerState};
use common::ws_common::ClientToServer;

// todo: incomplete
pub fn ui(ui: &mut Ui, state: &State) {
    let Some(ref timer) = state.timer else {return};

    ui.vertical_centered(|ui| {
        // title
        main_title(ui, &timer.state);

        time_progress_bar(ui, &timer.state.progress);

        buttons(ui, state, timer);
    });
}

fn main_title(ui: &mut Ui, timer_state: &TimerState) {
    let visuals = &ui.style().visuals;
    let (color, title) = match &timer_state.progress {
        PeriodProgress::Paused { .. } => (visuals.weak_text_color(), "Paused"),
        _ => match &timer_state.period {
            PeriodType::Uninit => (visuals.error_fg_color, "Uninit"),
            PeriodType::Work => (visuals.strong_text_color(), "Work"),
            PeriodType::Starting => (visuals.weak_text_color(), "Starting in"),
            PeriodType::ShortBreak => (visuals.text_color(), "Break"),
            PeriodType::LongBreak  => (visuals.text_color(), "Long Break"),
        }
    };
    ui.heading(RichText::new(title).color(color));
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
        let minutes = dur.num_minutes()%60;
        let hours = dur.num_hours();

        if hours > 0 {
            format!("{0}:{1:0>2}:{2:0>2}", hours, minutes, secs)
        } else {
            format!("{0:0>2}:{1:0>2}", minutes, secs)
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

        let stop_response = ui.add(Button::new("Stop").min_size(vec2(70.,1.)));
        if confirm_popup(ui, "stop_confirm_popup" , &stop_response) {
            state.ws_tx.send(ClientToServer::StopTimer).unwrap();
        }

        // skipping only makes sense if pomodoro is enabled
        if timer.profile.pomodoro.is_some() {
            let enabled = timer.profile.can_skip_work || !matches!(timer.state.period, PeriodType::Work);
            let skip_response = ui.add_enabled(enabled, Button::new("Skip").min_size(vec2(70.,1.)));
            if enabled && confirm_popup(ui, "skip_confirm_popup",&skip_response) {
                state.ws_tx.send(ClientToServer::SkipPeriod).unwrap();
            }
        }
    });
}

fn confirm_popup(ui: &mut Ui, id_source: impl Hash, response: &Response) -> bool {
    let popup_id = ui.id().with(id_source);
    if response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }

    popup_below_widget(ui, popup_id, response, |ui| {
        ui.set_min_width(90.);
        ui.label("Are you sure?");
        ui.button("Confirm").clicked()
    }).unwrap_or_default()
}