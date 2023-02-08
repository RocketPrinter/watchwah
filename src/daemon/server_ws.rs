use crate::common::ws_common::ClientToServer;
use axum::extract::ws::{Message, WebSocket};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};
use ClientToServer::*;
use crate::daemon::SState;

#[instrument(name = "server ws", skip_all)]
pub async fn handle_socket(mut ws: WebSocket, state: SState, mut rx: Receiver<String>) {
    info!("Connection established"); // todo: log ip/other info2x
    loop {
        select! {
            // message was received from websocket
            Some(received) = ws.recv() =>
                match received {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientToServer>(&text) {
                            Ok(msg) => handle_msg(&state, msg).await,
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

async fn send_welcome() {
    todo!()
}

async fn handle_msg(state: &SState, msg: ClientToServer) {
    if let Multiple(msgs) = msg {
        for msg in msgs {
            handle_msg_internal(state,msg).await;
        }
    } else {
        handle_msg_internal(state,msg).await;
    }

    async fn handle_msg_internal(state: &SState, msg: ClientToServer) {
        match msg {
            SetActiveProfile(name) => {
                let conf = state.conf.read().await;
                *state.current_profile.write().await = name.and_then(|name| conf.profiles.iter().find(|p|p.name == name).cloned() );

                let mut token = state.cancel_if_profile_changes.lock().await;
                token.cancel();
                *token = CancellationToken::new();
            },

            Create(_) => todo!(),
            PauseTimer => todo!(),
            UnpauseTimer => todo!(),
            StopTimer => todo!(),

            Multiple(_) => panic!(),
        }
    }

}