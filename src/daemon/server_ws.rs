use axum::extract::ws::{WebSocket, Message};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tracing::{info, error, instrument};
use crate::common::ws_common::{ClientToServer, ServerToClient};
use ClientToServer::*;

#[instrument(name = "server ws", skip_all)]
pub async fn handle_socket(mut ws: WebSocket, mut rx: Receiver<ServerToClient>) {
    info!("Connection established"); // todo: log ip/other info
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
                match rez.map(|msg| serde_json::to_string(&msg) ) {
                    Ok(Ok(text)) => if let Err(e) = ws.send(Message::Text(text)).await {
                        error!("Failed to send message: {e}");
                        return; // todo: does an error here mean that the websocket is closed?
                    },
                    Ok(Err(e)) => error!("Failed to serialize message: {e}"),
                    Err(e) => error!("Broadcast error: {e}"),
                }
        }
    }
}

fn handle_msg(/* todo: more fields */msg: ClientToServer) {
    match msg {
        SetActiveProfile(_) => todo!(),

        PauseTimer => todo!(),
        StopTimer => todo!(),

        Multiple(msgs) =>
            for msg in msgs {
                handle_msg(msg);
            },
    }
}
