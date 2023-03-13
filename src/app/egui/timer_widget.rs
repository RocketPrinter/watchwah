use eframe::egui::Ui;
use crate::app::State;
use crate::common::ws_common::ClientToServer;

pub fn ui(ui: &mut Ui, state: &State) {
    ui.label(format!("{:#?}", state.timer.as_ref().unwrap()));

    if ui.button("Unpause").clicked() {
        state.ws_tx.send(ClientToServer::UnpauseTimer).unwrap();
    }

    if ui.button("Pause").clicked() {
        state.ws_tx.send(ClientToServer::PauseTimer).unwrap();
    }

    if ui.button("Stop").clicked() {
        state.ws_tx.send(ClientToServer::StopTimer).unwrap();
    }
}