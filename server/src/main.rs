mod server_config;
mod server_ws;
mod timer_logic;

use std::net::SocketAddr;
use common::timer::Timer;
use common::ws_common::ServerToClient;
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::routing::get;
use axum::Router;
use std::process;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{broadcast, Mutex, Notify, RwLock};
use tracing::{error};
use common::register_tracing;
use crate::server_config::ServerConfig;

pub type SState = Arc<State>;
pub struct State {
    pub ws_tx: UnboundedSender<ServerToClient>,

    pub conf: RwLock<ServerConfig>,

    pub timer: Mutex<Option<Timer>>,
    pub cancel_timer_tasks: Arc<Notify>,
}

// todo: tracing

#[tokio::main]
pub async fn main() {
    register_tracing("127.0.0.1:6669");

    // state
    let (ws_tx, _ws_rx) = broadcast::channel::<String>(16);

    let state = Arc::new(State {
        ws_tx: server_ws::serialize_incoming(ws_tx.clone()),

        conf: match server_config::load_config() {
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
    let monitor = server_config::config_monitor(state.clone());
    tokio::spawn(async {
        if let Err(e) = monitor.await {
            error!("Monitor service failed: {e}");
        }
    });

    // axum
    let router = Router::new()
        // todo: rest
        .route(
            "/ws",
            get(move |upgrade: WebSocketUpgrade, ip: ConnectInfo<SocketAddr>| async move {
                upgrade.on_upgrade(move |ws| {
                    server_ws::handle_socket(ws, ip.0, state.clone(), ws_tx.subscribe())
                })
            }),
        );

    let server =
        axum::Server::bind(&"127.0.0.1:63086".parse().unwrap()).serve(router.into_make_service_with_connect_info::<SocketAddr>());

    if let Err(e) = server.await {
        error!("[Server] Axum failed with {e}")
    }
}
