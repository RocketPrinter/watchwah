mod egui_app;
mod websocket;

use std::sync::{Arc, Mutex};
use clap::builder::Str;
use tokio::runtime::EnterGuard;
use websockets::{Frame};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tracing::{Level, span};

pub type SState = Arc<Mutex<State>>;
#[derive(Debug)]
pub struct State {
    pub ws_tx: UnboundedSender<String>,
}

pub fn app() {
    // state
    let (ws_tx,ws_rx) = unbounded_channel::<String>();
    let state = Arc::new(Mutex::new(State{
        ws_tx,
    }));

    // websocket
    let sc = state.clone();
    tokio::spawn(async {websocket::ws_loop( sc, ws_rx).await });

    // egui on main thread
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Watchwah", native_options, Box::new(|cc| Box::new(egui_app::EguiApp::new(cc, state))));
}