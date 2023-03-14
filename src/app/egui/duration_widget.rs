use chrono::Duration;
use eframe::egui::{DragValue, Response, Ui};

pub fn ui(ui: &mut Ui, dur: &mut Duration) -> Response {
    ui.horizontal(|ui| {
        // warning we will lose sub second precision
        let mut minutes = dur.num_minutes() % 60;
        let mut hours = dur.num_hours();

        ui.add(DragValue::new(&mut hours).clamp_range(0..=999).suffix("h"));
        ui.add(DragValue::new(&mut minutes).clamp_range(if hours == 0 {1..=59} else {0..=59}).suffix("m"));

        *dur = Duration::hours(hours) + Duration::minutes(minutes);
    })
    .response
}
