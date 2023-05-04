mod egui;
mod client_ws;
mod detection;
mod client_config;
mod audio_manager;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use eframe::egui::Context;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Notify;
use common::register_tracing;
use crate::client_config::ClientConfig;
use common::timer::Timer;
use common::ws_common::{ClientToServer, ProfileInfo};
use anyhow::Result;
use crate::audio_manager::AudioManager;

pub type SState = Arc<Mutex<State>>;
pub struct State {
    pub config: ClientConfig,

    pub profiles: Vec<ProfileInfo>,
    pub timer: Option<Box<Timer>>,
    pub timer_updated: Arc<Notify>,

    pub ws_connected: bool,
    pub ws_tx: UnboundedSender<ClientToServer>,
    pub egui_context: Option<Context>,
    pub audio_manager: AudioManager,

    // for use in the secret debug menu
    //                              \/ title               \/ blocked    \/ extra info
    pub detected_windows: HashMap<String, (DateTime<Utc>, bool, Option<Vec<String>>)>,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    register_tracing("127.0.0.1:6670");

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
        audio_manager: AudioManager::new()?,

        detected_windows: HashMap::new(),
    }));

    // websocket
    let sc = state.clone();
    tokio::spawn(async { client_ws::ws_loop(sc, ws_rx).await });

    // detection
    let sc = state.clone();
    tokio::spawn(async{detection::blocker_loop(sc).await});

    // egui
    egui::run(state);

    Ok(())
}

//todo: sooound
//todo: toast popup
//todo: notifications when a period is over/starting
//todo: skip period button?
//todo: finish the ui
//todo: finish x11 detection
// clean up code
// sfx
// actual detection??
// switch away from notifications?
//todo: wayland + windows detection
//todo: notify 1 min before end of period
//todo: if not using pomodoro have a button that starts a break?