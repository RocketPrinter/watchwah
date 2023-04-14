use chrono::{Utc};
use crate::State;
use eframe::egui::{Align, CollapsingHeader, Color32, Context, Label, Layout, popup_below_widget, Response, RichText, ScrollArea, Sense, TextStyle, TopBottomPanel, Ui};
use eframe::egui::special_emojis::GITHUB;
use crate::audio_manager::SoundEffects;

pub fn ui(ctx: &Context, state: &State) {
    TopBottomPanel::top("top")
        .min_height(0.)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let logo_response = ui.selectable_label(false, RichText::new("Watchwah").text_style(TextStyle::Heading).size(20.));
                popup(ui, &logo_response, state);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.hyperlink_to(GITHUB.to_string(), "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
                });
            });
        });
}

fn popup(ui: &mut Ui, logo_response: &Response, state: &State) {
    let popup_id = ui.id().with("logo_popup");
    let updates = ui.memory_mut(|mem| {
        // open popup if logo is clicked
        if logo_response.clicked() {
            mem.toggle_popup(popup_id);
        }

        // nr of updates
        let updates = mem.data.get_temp_mut_or_default::<u32>(ui.id());
        *updates+=1;
        *updates
    });

    let popup_width = ui.available_width();
    popup_below_widget(ui, popup_id, logo_response, |ui| {
        ui.set_max_width(popup_width);
        if ui.add(Label::new("Welcome to the secret menu!").sense(Sense::click())).clicked() {
            state.audio_manager.play_logged(SoundEffects::Secret);
        }
        ui.label(format!("Connected: {0}", state.ws_connected));
        ui.label(format!("Updates: {0}", updates));

        CollapsingHeader::new("Detected windows").default_open(true).show(ui, |ui| {
            let utc = Utc::now();
            for (name, (time, blocked, extra)) in state.detected_windows.iter() {
                ui.label(RichText::new(format!("  {}s ago: {name}", (utc - *time).num_seconds())).color(if *blocked {Color32::RED} else {Color32::LIGHT_GRAY}));
                for str in extra.iter().flat_map(|v|v.iter()) {
                    ui.label(str);
                }
            }
        });

        CollapsingHeader::new("Timer dbg!").default_open(true).show(ui, |ui| {
            match state.timer.as_ref() {
                None => {ui.label("None");},
                Some(timer) => {
                    ScrollArea::new([false,true]).max_height(250.).show(ui, |ui| {
                        ui.label(format!("{:#?}", timer));
                    });
                },
            }
        });
    });
}
