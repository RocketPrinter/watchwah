mod egui;
mod client_ws;
mod blocking;
mod client_config;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use eframe::egui::Context;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use crate::app::client_config::ClientConfig;
use crate::common::timer::Timer;
use crate::common::ws_common::ClientToServer;

pub type SState = Arc<Mutex<State>>;
#[derive(Debug)]
pub struct State {
    pub config: ClientConfig,

    pub profiles: Vec<String>,
    pub timer: Option<Timer>,

    pub ws_connected: bool,
    pub ws_tx: UnboundedSender<ClientToServer>,
    pub egui_context: Option<Context>,
}

pub fn app() {
    // state
    let (ws_tx,ws_rx) = unbounded_channel::<ClientToServer>();
    let state = Arc::new(Mutex::new(State{
        config: client_config::load_config(),

        profiles: vec![],
        timer: None,
        
        ws_connected: false,
        ws_tx,
        egui_context: None,
    }));

    // websocket
    let sc = state.clone();
    tokio::spawn(async { client_ws::ws_loop(sc, ws_rx).await });

    // egui
    egui::run(state);
}