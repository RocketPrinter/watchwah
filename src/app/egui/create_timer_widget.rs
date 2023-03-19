use std::sync::Arc;
use crate::app::egui::duration_input_widget;
use crate::app::State;
use crate::common::timer::TimerGoal;
use crate::common::ws_common::ClientToServer;
use chrono::Duration;
use eframe::egui::{Button, ComboBox, Ui};
use eframe::egui::mutex::Mutex;

pub struct CreateTimerState {
    pub selected_profile: Option<String>,
    pub selected_goal: TimerGoal,
}

impl Default for CreateTimerState {
    fn default() -> Self {
        Self {
            selected_profile: None,
            selected_goal: TimerGoal::None,
        }
    }
}

pub fn ui(ui: &mut Ui, state: &State) {
    let state_id = ui.id().with("create_timer");
    let data = ui.data_mut(|d| {
        d.get_temp_mut_or_insert_with(state_id, || Arc::new(Mutex::new( CreateTimerState::default()))).clone()
    });
    let mut data = data.lock();

    // Profile
    ui.horizontal(|ui| {
        // in case the profile doesn't exist anymore
        if data
            .selected_profile
            .as_ref()
            .map(|p| state.profiles.contains(p))
            == Some(false)
        {
            data.selected_profile = None;
        }
        ui.label("Profile:");

        ComboBox::from_id_source("profile_select")
            .selected_text(
                data.selected_profile
                    .as_ref()
                    .map(|s| &s[..])
                    .unwrap_or("Select a profile"),
            )
            .show_ui(ui, |ui| {
                for profile in &state.profiles {
                    ui.selectable_value(&mut data.selected_profile, Some(profile.clone()), profile);
                }
            })
    });

    // Goal
    ui.horizontal(|ui| {
        ui.label("Goal:");
        ui.selectable_value(&mut data.selected_goal, TimerGoal::None, "None");
        if ui
            .selectable_label(matches!(data.selected_goal, TimerGoal::Time(_)), "Time")
            .clicked()
        {
            data.selected_goal = TimerGoal::Time(Duration::minutes(15));
        }
        if ui
            .selectable_label(matches!(data.selected_goal, TimerGoal::Todos(_)), "Todo")
            .clicked()
        {
            data.selected_goal = TimerGoal::Todos(69);
        }
    });

    match data.selected_goal {
        TimerGoal::None => (),
        TimerGoal::Time(ref mut duration) => {
            ui.horizontal(|ui| {
                ui.label("Duration:");
                duration_input_widget::ui(ui, duration);
            });
        }
        TimerGoal::Todos(_count) => {
            ui.label("todo");
        }
    }

    // Start button
    if ui
        .add_enabled(data.selected_profile.is_some(), Button::new("Start"))
        .clicked()
    {
        state
            .ws_tx
            .send(ClientToServer::CreateTimer {
                profile_name: data.selected_profile.clone().unwrap(),
                goal: data.selected_goal.clone(),
            })
            .ok();
    }
}
