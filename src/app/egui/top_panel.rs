use crate::app::egui::EguiApp;
use crate::app::State;
use eframe::egui::{Align, Context, Layout, popup_below_widget, Response, RichText, TextStyle, TopBottomPanel, Ui};
use eframe::egui::special_emojis::GITHUB;
use tracing::error;
use tracing::instrument::WithSubscriber;

pub fn ui(ctx: &Context, state: &State) {
    TopBottomPanel::top("top")
        .min_height(0.)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let logo_response = ui.selectable_label(false, RichText::new("Watchwah").text_style(TextStyle::Heading).size(20.));
                popup(ui, &logo_response, &state);

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
        ui.label("Welcome to the secret menu!").on_hover_text("OwO");
        ui.label(format!("Connected: {0}", state.ws_connected));
        ui.label(format!("Updates: {0}", updates));
        if ui.button("X11 test").clicked() {
            if let Err(e) = crate::app::blocking::x11::proof_of_concept() {
                error!("Error: {}", e);
            }
        }
    });
}
