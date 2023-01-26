mod server_ws;
mod rest;
mod config_service;
mod timer_service;

use axum::extract::WebSocketUpgrade;
use axum::Router;
use axum::routing::get;
use tokio::sync::broadcast;
use tracing::error;
use crate::common::ws_common::ServerToClient;

pub fn daemon() {
    let (ws_tx, _ws_rx) = broadcast::channel::<ServerToClient>(16);

    // config

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