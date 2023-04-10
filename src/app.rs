mod egui;
mod client_ws;
mod blocking;
mod client_config;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::{DateTime, Utc};
use eframe::egui::Context;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Notify;
use crate::app::client_config::ClientConfig;
use crate::common::timer::Timer;
use crate::common::ws_common::ClientToServer;

pub type SState = Arc<Mutex<State>>;
#[derive(Debug)]
pub struct State {
    pub config: ClientConfig,

    pub profiles: Vec<String>,
    pub timer: Option<Timer>,
    pub timer_updated: Arc<Notify>,

    pub ws_connected: bool,
    pub ws_tx: UnboundedSender<ClientToServer>,
    pub egui_context: Option<Context>,

    // for use in the secret debug menu
    //                              \/ title               \/ blocked    \/ extra info
    pub detected_windows: HashMap<String, (DateTime<Utc>, bool, Option<Vec<String>>)>,
}

pub fn app() {
    // state
    let (ws_tx,ws_rx) = unbounded_channel::<ClientToServer>();
    let state = Arc::new(Mutex::new(State{
        config: client_config::load_config(),

        profiles: vec![],
        timer: None,
        timer_updated: Arc::new(Notify::new()),

        ws_connected: false,
        ws_tx,
        egui_context: None,

        detected_windows: HashMap::new(),
    }));

    // websocket
    let sc = state.clone();
    tokio::spawn(async { client_ws::ws_loop(sc, ws_rx).await });

    // blocking
    let sc = state.clone();
    tokio::spawn(async{blocking::blocker_loop(sc).await});

    // egui
    egui::run(state);
}

//todo: sooound
//todo: toast popup
//todo: finish the ui
//todo: finish x11 blocking
    // clean up code
    // sounds
    // actual blocking??
    // switch away from notifications?
//todo: wayland + windows blocking