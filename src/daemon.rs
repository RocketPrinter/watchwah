mod server_ws;
mod rest;
mod config_service;
mod timer_service;

use std::process;
use std::sync::Arc;
use axum::extract::{WebSocketUpgrade};
use axum::Router;
use axum::routing::get;
use tokio::sync::{broadcast, Mutex, Notify, RwLock};
use tokio::sync::broadcast::Sender;
use tracing::{error, instrument};
use crate::common::config::ServerConfig;
use crate::common::timer::Timer;
use crate::common::ws_common::ServerToClient;

pub type SState = Arc<State>;
pub struct State {
    pub ws_tx: Sender<ServerToClient>,

    pub conf: RwLock<ServerConfig>,

    pub timer: Mutex<Option<Timer>>,
    pub cancel_timer_tasks: Arc<Notify>,
}

#[instrument(name="daemon", skip_all)]
pub async fn daemon() {
    // state
    let (ws_tx, _ws_rx) = broadcast::channel::<ServerToClient>(16);

    let state = Arc::new(State{
        ws_tx: ws_tx.clone(),

        conf: match config_service::load() {
            Ok(conf) => RwLock::new(conf),
            Err(err) => {error!("Unable to load config: {err}"); process::exit(-1)},
        },
        timer: Mutex::new(None),
        cancel_timer_tasks: Arc::new(Notify::new()),
    });

    // config monitor
    let monitor = config_service::config_monitor(state.clone());
    tokio::spawn(async {
        if let Err(e) = monitor.await { error!("Monitor service failed: {e}");}
    });

    // timer service




    // axum
    let router = Router::new()
        // todo: rest
        .route("/ws", get( move |upgrade: WebSocketUpgrade|
            async {upgrade.on_upgrade(move |ws| server_ws::handle_socket(ws, state.clone(),ws_tx.subscribe()))}
        ));

    let server = axum::Server::bind(&"127.0.0.1:63086".parse().unwrap())
        .serve(router.into_make_service());

    tokio::spawn(async {
        if let Err(e) = server.await {
            error!("[Server] Axum failed with {e}")
        }
    });
}