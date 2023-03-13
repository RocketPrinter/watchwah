mod config_logic;
mod rest;
mod server_ws;
mod timer_logic;

use crate::common::config::ServerConfig;
use crate::common::timer::Timer;
use crate::common::ws_common::ServerToClient;
use axum::extract::WebSocketUpgrade;
use axum::routing::get;
use axum::Router;
use std::process;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{broadcast, Mutex, Notify, RwLock};
use tracing::{error, instrument};

pub type SState = Arc<State>;
pub struct State {
    pub ws_tx: UnboundedSender<ServerToClient>,

    pub conf: RwLock<ServerConfig>,

    pub timer: Mutex<Option<Timer>>,
    pub cancel_timer_tasks: Arc<Notify>,
}

#[instrument(name = "daemon", skip_all)]
pub fn daemon() {
    // state
    let (ws_tx, _ws_rx) = broadcast::channel::<String>(16);

    let state = Arc::new(State {
        ws_tx: server_ws::serialize_incoming(ws_tx.clone()),

        conf: match config_logic::load() {
            Ok(conf) => RwLock::new(conf),
            Err(err) => {
                error!("Unable to load config: {err}");
                process::exit(-1)
            }
        },
        timer: Mutex::new(None),
        cancel_timer_tasks: Arc::new(Notify::new()),
    });

    // config monitor
    let monitor = config_logic::config_monitor(state.clone());
    tokio::spawn(async {
        if let Err(e) = monitor.await {
            error!("Monitor service failed: {e}");
        }
    });

    // timer service

    // axum
    let router = Router::new()
        // todo: rest
        .route(
            "/ws",
            get(move |upgrade: WebSocketUpgrade| async {
                upgrade.on_upgrade(move |ws| {
                    server_ws::handle_socket(ws, state.clone(), ws_tx.subscribe())
                })
            }),
        );

    let server =
        axum::Server::bind(&"127.0.0.1:63086".parse().unwrap()).serve(router.into_make_service());

    tokio::spawn(async {
        if let Err(e) = server.await {
            error!("[Server] Axum failed with {e}")
        }
    });
}
