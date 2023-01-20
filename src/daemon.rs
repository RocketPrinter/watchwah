mod server_ws;
mod rest;

use std::sync::{Arc, Mutex};
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::WebSocket;
use axum::Router;
use axum::ServiceExt;
use axum::routing::get;
use tracing::error;

pub type SState = Arc<Mutex<State>>;
#[derive(Debug)]
pub struct State {

}

pub fn daemon() {
    let mut state = Arc::new(Mutex::new(State{

    }));

    // server
    let router = Router::new()
        .route("/ws", get( |ws: WebSocketUpgrade| async {ws.on_upgrade(handle_socket)}))
        .with_state(state.clone());

    let server = axum::Server::bind(&"0.0.0.0:63086".parse().unwrap())
        .serve(router.into_make_service());

    tokio::spawn(async {
        if let Err(e) = server.await {
            error!("[Server] Axum failed with {e}")
        }
    });
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}