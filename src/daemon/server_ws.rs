use crate::common::ws_common::ClientToServer;
use axum::extract::ws::{Message, WebSocket};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tracing::{error, info, instrument};
use ClientToServer::*;

#[instrument(name = "server ws", skip_all)]
pub async fn handle_socket(mut ws: WebSocket, mut rx: Receiver<String>) {
    info!("Connection established"); // todo: log ip/other info2x
    loop {
        select! {
            // message was received from websocket
            Some(received) = ws.recv() =>
                match received {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientToServer>(&text) {
                            Ok(msg) => handle_msg(msg),
                            Err(e) => error!("Failed to deserialize message: {e}"),
                        }
                    },
                    Ok(_) => (),
                    Err(e) => {
                        error!("Failed to receive message: {e}");
                        return; // todo: does an error here mean that the websocket is closed?
                    }
                },
            // message was received from broadcast
            rez = rx.recv() =>
                match rez {
                    Ok(text) => if let Err(e) = ws.send(Message::Text(text)).await {
                        error!("Failed to send message: {e}");
                        return; // todo: does an error here mean that the websocket is closed?
                    },
                    Err(e) => error!("Broadcast error: {e}"),
                }
        }
    }
}

fn handle_msg(/* todo: more fields */ msg: ClientToServer) {
    match msg {
        SetActiveProfile(_) => todo!(),

        Create(_) => todo!(),
        PauseTimer => todo!(),
        UnpauseTimer => todo!(),
        StopTimer => todo!(),

        Multiple(msgs) => {
            for msg in msgs {
                handle_msg(msg);
            }
        }
    }
}
