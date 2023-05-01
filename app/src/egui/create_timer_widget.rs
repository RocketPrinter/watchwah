use std::sync::Arc;
use crate::egui::{duration_input_widget, TOMATO};
use crate::State;
use common::timer::TimerGoal;
use common::ws_common::{ClientToServer, ProfileInfo};
use chrono::Duration;
use eframe::egui::{Button, ComboBox, DragValue, Ui, vec2, Widget};
use eframe::egui::mutex::Mutex;
use crate::egui::centerer::centerer;

#[derive(Clone, Debug, Default)]
pub struct CreateTimerState {
    pub selected_profile: Option<ProfileInfo>,
    pub selected_goal: TimerGoal,
    pub start_in: Option<Duration>,
}

pub fn ui(ui: &mut Ui, state: &State) {
    let state_id = ui.id().with("create_timer");
    let data = ui.data_mut(|d| {
        d.get_temp_mut_or_insert_with(state_id, || Arc::new(Mutex::new( CreateTimerState::default()))).clone()
    });
    // we have to deref to avoid borrow checker weirdness
    let data = &mut *data.lock();

    // ------ profile ------
    ui.horizontal(|ui| {
        // in case the profile doesn't exist anymore
        if data
            .selected_profile
            .as_ref()
            .map(|s| state.profiles.iter().any(|p| p == s))
            == Some(false)
        {
            data.selected_profile = None;
        }
        ui.label("Profile:");

        // combo box
        let text = data.selected_profile
            .as_ref()
            .map(format_profile_info)
            .unwrap_or("Select a profile".to_string());
        ComboBox::from_id_source("profile_select")
            .selected_text(text)
            .show_ui(ui, |ui| {
                for profile in &state.profiles {
                    ui.selectable_value(&mut data.selected_profile, Some(profile.clone()), format_profile_info(profile));
                }
            })
    });

    // ------ goal ------
    ui.horizontal(|ui| {
        ui.label("Goal:");
        ui.selectable_value(&mut data.selected_goal, TimerGoal::None, "None");
        if ui
            .selectable_label(matches!(data.selected_goal, TimerGoal::Time(_)), "Time")
            .clicked()
        {
            let default_dur = data.selected_profile.as_ref().and_then(|p|p.pomo_work_dur).unwrap_or_else(||Duration::minutes(15));
            data.selected_goal = TimerGoal::Time(default_dur);
        }
        if ui
            .selectable_label(matches!(data.selected_goal, TimerGoal::Todos(_)), "Todo")
            .clicked()
        {
            data.selected_goal = TimerGoal::Todos(69);
        }
    });

    // goal specific settings
    match data.selected_goal {
        TimerGoal::None => (),
        TimerGoal::Time(ref mut duration) => {
            ui.horizontal(|ui| {
                ui.label("Duration:");
                duration_input_widget::ui(ui, duration);

                if let Some(ProfileInfo{ pomo_work_dur: Some(work_dur), ..}) = data.selected_profile {
                    let mut pomodoros = duration.num_seconds() as f32 / work_dur.num_seconds() as f32;
                    let mut new_pomodoros = pomodoros;
                    ui.label(" or ");
                    if DragValue::new(&mut new_pomodoros).speed(1).suffix(TOMATO).ui(ui).changed() {
                        if new_pomodoros.fract() == 0. {
                            pomodoros = new_pomodoros;
                        } else {
                            // we round the number up or down
                            pomodoros = if new_pomodoros > pomodoros {
                                new_pomodoros.ceil()
                            } else {
                                new_pomodoros.floor()
                            };
                        }
                        // calculate new duration and check bounds
                        let new_dur = work_dur * (pomodoros as i32);
                        if new_dur.num_minutes() >= 1 &&  new_dur.num_hours() <= 999 {
                            *duration = new_dur;
                        }
                    }
                    // todo: show total time including breaks
                }
            });
        }
        TimerGoal::Todos(_count) => {
            ui.label("todo"); // todo
        }
    }

    // ------ start in ------
    ui.horizontal(|ui| {
        ui.label("Start:");
        ComboBox::from_id_source("start_in")
            .selected_text(if data.start_in.is_some() {"In"} else {"Now"})
            .width(50.)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut data.start_in, None, "Now");
                if ui.selectable_label(data.start_in.is_some(), "In").clicked() { data.start_in = Some(Duration::minutes(5));}
            });
        if let Some(ref mut start_in) = data.start_in {
            duration_input_widget::ui(ui, start_in);
        }
    });

    // ------ Start button ------
    if ui
        .add_enabled(data.selected_profile.is_some(), Button::new("Start"))
        .clicked()
    {
        state
            .ws_tx
            .send(ClientToServer::CreateTimer {
                profile_name: data.selected_profile.clone().unwrap().name,
                goal: data.selected_goal.clone(),
                start_in: data.start_in,
            })
            .ok();
    }
}

fn format_profile_info(pi: &ProfileInfo) -> String {
    format!("{}{}", pi.name, if pi.pomo_work_dur.is_some() { TOMATO } else { '\0' })
}