use chrono::Duration;
use eframe::egui::{Align, Button, Layout, ProgressBar, RichText, Ui, vec2, Widget};
use core::time::Duration as StdDuration;
use crate::egui::helpers::{centerer, confirm_popup, TOMATO};
use crate::State;
use common::timer::{PeriodProgress, PeriodType, Timer, TimerGoal, TimerState};
use common::ws_common::ClientToServer;

// todo: incomplete
pub fn ui(ui: &mut Ui, state: &State) {
    let Some(ref timer) = state.timer else {return};

    ui.vertical_centered(|ui| {

        main_title(ui, &timer.state);

        time_progress_bar(ui, &timer.state.progress);

        goal_info(ui, timer);

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

fn goal_info(ui: &mut Ui, timer: &Timer) {
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        match timer.goal {
            TimerGoal::None => {}
            TimerGoal::Time(dur) => {
                let Some(ref pomodoro) = timer.profile.pomodoro else {return};
                // display number of pomodoros
                ui.label(format!("{}/{}{TOMATO}",
                                 pomodoro.calc_pomodoros(timer.state.total_dur_worked),
                                 pomodoro.calc_pomodoros(dur)
                ));
            }
            TimerGoal::Todos(_) => {
                ui.label("Todo!"); // todo: display number of todos
            }
        }
    });
}

fn buttons(ui: &mut Ui, state: &State, timer: &Timer) {
    centerer(ui, |ui| {
        if let PeriodProgress::Running {..} = timer.state.progress {
            if timer.profile.can_pause && Button::new("Pause").min_size(vec2(70.,1.)).ui(ui).clicked() {
                state.ws_tx.send(ClientToServer::PauseTimer).unwrap();
            }
        } else if ui.add(Button::new("Unpause").min_size(vec2(70.,1.))).clicked() {
            state.ws_tx.send(ClientToServer::UnpauseTimer).unwrap();
        }

        // todo: let enabled = timer.profile.can_stop_before_goal_is_fulfilled;
        let stop_response = ui.add_enabled(true ,Button::new("Stop").min_size(vec2(70.,1.)));
        if confirm_popup(ui, "stop_confirm_popup" , &stop_response) {
            state.ws_tx.send(ClientToServer::StopTimer).unwrap();
        }

        // skipping only makes sense if pomodoro is enabled or the period is a starting break
        if timer.profile.pomodoro.is_some() || matches!(timer.state.period, PeriodType::Starting) {
            let enabled = timer.profile.can_skip_work || !matches!(timer.state.period, PeriodType::Work);
            let skip_response = ui.add_enabled(enabled, Button::new("Skip").min_size(vec2(70.,1.)));
            if enabled && confirm_popup(ui, "skip_confirm_popup",&skip_response) {
                state.ws_tx.send(ClientToServer::SkipPeriod).unwrap();
            }
        }
    });
}
