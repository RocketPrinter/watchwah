mod egui_app;
mod client_ws;

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use crate::common::profile::Profile;
use crate::common::ws_common::ClientToServer;

pub type SState = Arc<Mutex<State>>;
#[derive(Debug)]
pub struct State {
    pub active_profile: Option<Profile>,
    pub profiles: Vec<String>,

    pub ws_tx: UnboundedSender<ClientToServer>,
}

pub fn app() {
    // state
    let (ws_tx,ws_rx) = unbounded_channel::<ClientToServer>();
    let state = Arc::new(Mutex::new(State{
        active_profile: None,
        profiles: vec![],

        ws_tx,
    }));

    // websocket
    let sc = state.clone();
    tokio::spawn(async { client_ws::ws_loop(sc, ws_rx).await });

    // egui on main thread
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Watchwah", native_options, Box::new(|cc| Box::new(egui_app::EguiApp::new(cc, state))));
}