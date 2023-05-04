use std::hash::Hash;
use eframe::egui::{DragValue, popup_below_widget, Response, Ui, Widget};
use chrono::Duration;

pub const TOMATO: char = match char::from_u32(0x1F345) {
    Some(c) => c,
    None => panic!(),
};

// Helper function to center arbitrary widgets. It works by measuring the width of the widgets after rendering, and
// then using that offset on the next frame.
pub fn centerer(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        let id = ui.id().with("_centerer");
        let last_width: Option<f32> = ui.memory_mut(|mem| mem.data.get_temp(id));
        if let Some(last_width) = last_width {
            ui.add_space((ui.available_width() - last_width) / 2.0);
        }
        let res = ui
            .scope(|ui| {
                add_contents(ui);
            })
            .response;
        let width = res.rect.width();
        ui.memory_mut(|mem| mem.data.insert_temp(id, width));

        // Repaint if width changed
        match last_width {
            None => ui.ctx().request_repaint(),
            Some(last_width) if last_width != width => ui.ctx().request_repaint(),
            Some(_) => {}
        }
    });
}

pub fn confirm_popup(ui: &mut Ui, id_source: impl Hash, response: &Response) -> bool {
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

pub fn duration_input_widget(ui: &mut Ui, dur: &mut Duration) -> Response {
    ui.horizontal(|ui| {
        // warning we will lose sub second precision
        let mut hours = dur.num_hours();
        let mut minutes = dur.num_minutes() % 60;

        DragValue::new(&mut hours).clamp_range(0..=999).suffix("h").ui(ui);
        ui.add_space(-6.);
        DragValue::new(&mut minutes).clamp_range(if hours == 0 {1..=59} else {0..=59}).suffix("m").ui(ui);

        *dur = Duration::hours(hours) + Duration::minutes(minutes);
    })
    .response
}
