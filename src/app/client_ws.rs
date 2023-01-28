use std::time::Duration;
use tokio::{select};
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::time::sleep;
use tracing::{error, info, instrument};
use websockets::{Frame, WebSocket, WebSocketError};
use anyhow::Result;
use crate::app::SState;
use crate::common::ws_common::{ClientToServer, ServerToClient};

const URL: &str = "ws://localhost:63086/ws";

/// Handles reconnections and message processing
#[instrument(name = "client ws", skip_all)]
pub async fn ws_loop(state: SState, mut rx: UnboundedReceiver<ClientToServer>) {
    loop {
        let e = match WebSocket::connect(URL).await {
            Ok(ws) => {
                info!("Connection established");
                select_loop(&state, ws,&mut rx).await.unwrap_err()
            },
            Err(err) => err,
        };
        error!("Stopped with error \"{0}\". Retrying in 3 seconds ", e.to_string());

        // wait 3 seconds before retrying
        sleep(Duration::from_secs(3)).await;
    }
}

async fn select_loop(state: &SState, mut ws: WebSocket, rx: &mut UnboundedReceiver<ClientToServer>) -> Result<(),WebSocketError> {
    let mut incomplete_payload: Option<String> = None;
    loop {
        select! {
            // message was received from the websocket that needs to be handled
            received = ws.receive() => {
                if let Frame::Text { payload: frame_payload, fin, .. } = received? {

                    let payload = if let Some(p) = incomplete_payload.take() { p + &frame_payload } else {frame_payload};

                    if fin {
                        // message has finalized, we can handle it
                        match serde_json::from_str::<ServerToClient>(&payload) {
                            Ok(payload) => handle_msg(state,payload),
                            Err(e) => error!("[Client WS] Failed to deserialize message: {e}")
                        }
                    } else {
                        // message is not finalized yet
                        incomplete_payload = Some(payload);
                    }
                }
            },
            // message was received from channel that needs to be sent to the ws
            Some(msg) = rx.recv() =>
                match serde_json::to_string(&msg) {
                    Ok(text) => ws.send(Frame::text(text)).await?,
                    Err(e) => error!("[Client WS] Failed to serialize message: {e}"),
                },
        }
    }
}

fn handle_msg(state: &SState, msg: ServerToClient) {
    match msg {
        ServerToClient::UpdateProfiles(profiles) => {
            state.lock().unwrap().profiles = profiles;
        }
        ServerToClient::UpdateActiveProfile(profile) => {
            state.lock().unwrap().active_profile = profile;
        }

        ServerToClient::RefreshedConfig => {todo!()} // show a popup

        ServerToClient::Multiple(msgs) =>
            for msg in msgs {
            handle_msg(state, msg);
        }
    }
}