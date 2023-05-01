use chrono::Duration;
use eframe::egui::{DragValue, Response, Ui, Widget};

pub fn ui(ui: &mut Ui, dur: &mut Duration) -> Response {
    ui.horizontal(|ui| {
        // warning we will lose sub second precision
        let mut hours = dur.num_hours();
        let mut minutes = dur.num_minutes() % 60;

        DragValue::new(&mut hours).clamp_range(0..=999).suffix("h").ui(ui);
        DragValue::new(&mut minutes).clamp_range(if hours == 0 {1..=59} else {0..=59}).suffix("m").ui(ui);

        *dur = Duration::hours(hours) + Duration::minutes(minutes);
    })
    .response
}
