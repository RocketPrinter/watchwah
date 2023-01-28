mod server_ws;
mod rest;
mod config_service;
mod timer_service;

use std::sync::Arc;
use axum::extract::WebSocketUpgrade;
use axum::Router;
use axum::routing::get;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, instrument};
use crate::common::config;
use crate::common::config::Config;
use crate::common::ws_common::ServerToClient;

type SConfig = Arc<RwLock<Config>>;

#[instrument(name="daemon", skip_all)]
pub async fn daemon() {
    let (ws_tx, _ws_rx) = broadcast::channel::<ServerToClient>(16);

    // config
    let conf = config::load(&config::get_config_path())
        .map_err(|e| error!("Unable to load config: {e}") ).unwrap();
    let conf = Arc::new(RwLock::new(conf));
    let monitor = config_service::config_monitor(conf.clone(), ws_tx.clone());
    tokio::spawn(async {
        if let Err(e) = monitor.await { error!("Monitor service failed: {e}");}
    });

    // timer service

    // axum
    let router = Router::new()
        .route("/ws", get( move |upgrade: WebSocketUpgrade|
            async {upgrade.on_upgrade(move |ws| server_ws::handle_socket(ws,ws_tx.subscribe()))}
        ));
    // todo: rest

    let server = axum::Server::bind(&"0.0.0.0:63086".parse().unwrap())
        .serve(router.into_make_service());

    tokio::spawn(async {
        if let Err(e) = server.await {
            error!("[Server] Axum failed with {e}")
        }
    });
}