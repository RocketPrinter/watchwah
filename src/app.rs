mod egui_app;
mod client_ws;
//todo: mod client_config;

use std::sync::{Arc, Mutex};
use eframe::egui::Context;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use crate::common::profile::Profile;
use crate::common::timer_state::TimerState;
use crate::common::ws_common::ClientToServer;

pub type SState = Arc<Mutex<State>>;
#[derive(Debug)]
pub struct State {
    pub active_profile: Option<Profile>,
    pub profiles: Vec<String>,
    pub timer: TimerState,

    pub ws_connected: bool,
    pub ws_tx: UnboundedSender<ClientToServer>,
    pub egui_context: Option<Context>,
}

pub fn app() {
    // state
    let (ws_tx,ws_rx) = unbounded_channel::<ClientToServer>();
    let state = Arc::new(Mutex::new(State{
        active_profile: None,
        profiles: vec![],
        timer: TimerState::NotCreated,
        
        ws_connected: false,
        ws_tx,
        egui_context: None,
    }));

    // websocket
    let sc = state.clone();
    tokio::spawn(async { client_ws::ws_loop(sc, ws_rx).await });

    // egui
    egui_app::run(state);
}