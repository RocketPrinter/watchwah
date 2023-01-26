use axum::extract::ws::{WebSocket, Message};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tracing::{info,error};
use crate::common::ws_common::{ClientToServer, ServerToClient};

pub async fn handle_socket(/* todo: more fields*/mut ws: WebSocket, mut rx: Receiver<ServerToClient>) {
    info!("[Server WS] Connection established"); // todo: log ip/other info
    loop {
        select! {
            // message was received from websocket
            Some(received) = ws.recv() =>
                match received {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientToServer>(&text) {
                            Ok(msg) => handle_msg(msg),
                            Err(e) => error!("[Server WS] Failed to deserialize message: {e}"),
                        }
                    },
                    Ok(_) => (),
                    Err(e) => {
                        error!("[Server WS] Failed to receive message: {e}");
                        return; // todo: does an error here mean that the websocket is closed?
                    }
                },
            // message was received from broadcast
            rez = rx.recv() =>
                match rez.map(|msg| serde_json::to_string(&msg) ) {
                    Ok(Ok(text)) => if let Err(e) = ws.send(Message::Text(text)).await {
                        error!("[Server WS] Failed to send message: {e}");
                        return; // todo: does an error here mean that the websocket is closed?
                    },
                    Ok(Err(e)) => error!("[Server WS] Failed to serialize message: {e}"),
                    Err(e) => error!("[Server WS] Broadcast: {e}"),
                }
        }
    }
}

fn handle_msg(/* todo: more fields */msg: ClientToServer) {
    match msg {
        ClientToServer::SetActiveProfile(_) => todo!(),

        ClientToServer::RefreshConfig => todo!(),

        ClientToServer::Multiple(msgs) =>
            for msg in msgs {
                handle_msg(msg);
            },
    }
}
