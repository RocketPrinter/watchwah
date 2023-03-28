use std::net::SocketAddr;
use std::sync::Arc;
use crate::common::ws_common::{ClientToServer, ServerToClient};
use crate::daemon::{config_logic, SState, timer_logic};
use anyhow::{bail, Result};
use axum::extract::ws::{Message, WebSocket};
use tokio::select;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tracing::{error, info, instrument, warn};
use ClientToServer::*;

pub fn serialize_incoming(broadcast_tx: Sender<String>) -> UnboundedSender<ServerToClient>{
    let (tx,mut rx) = unbounded_channel::<ServerToClient>();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg = match serde_json::to_string(&msg) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to serialize message: {e}");
                    continue;
                }
            };
            if let Err(e) = broadcast_tx.send(msg) {
                error!("Failed to send message: {e}");
            }
        }
    });

    tx
}

#[instrument(name = "server ws", skip_all)]
pub async fn handle_socket(mut ws: WebSocket, ip: SocketAddr, state: SState, mut rx: Receiver<String>) {
    info!("Connection established. Ip: {ip}");

    if let Err(e) = send_welcome_message(&mut ws, &state).await {
        error!("Failed to send welcome message: {e}");
        return;
    }

    loop {
        select! {
            // message was received from websocket
            received = ws.recv() =>
                match received {
                    Some(Ok(Message::Text(msg))) => {
                        if let Err(e) = handle_receive(&state,msg).await {
                            error!("Error processing message: {e}")
                        }
                    },
                    Some(Ok(_)) => (),
                    Some(Err(e)) => {error!("Websocket errored: {e}"); return;}
                    None => {warn!("Websocket closed"); return;}, // stream closed
                },
            // message was received from broadcast
            rez = rx.recv() =>{
                match rez {
                    Ok(msg) => if let Err(e) = ws.send(Message::Text(msg)).await {
                        error!("Failed to send message: {e}");
                    },
                    Err(e) => error!("Broadcast error: {e}"),
                }}
        }
    }
}

async fn send_welcome_message(ws: &mut WebSocket, state: &SState) -> Result<()> {
    let msg = ServerToClient::Multiple(vec![
        config_logic::profiles_msg(state).await,
        ServerToClient::UpdateTimer(state.timer.lock().await.clone()),
    ]);

    ws.send(Message::Text(serde_json::to_string(&msg)?)).await?;
    Ok(())
}

async fn handle_receive(state: &SState, msg: String) -> Result<()> {
    let msg = serde_json::from_str(&msg)?;

    if let Multiple(msgs) = msg {
        for msg in msgs {
            handle_msg(state, msg).await?;
        }
    } else {
        handle_msg(state, msg).await?;
    }

    return Ok(());

    async fn handle_msg(state: &SState, msg: ClientToServer) -> Result<()> {
        let response = match msg {
            CreateTimer { goal, profile_name } => timer_logic::create_timer(state, goal, profile_name).await?,
            PauseTimer => timer_logic::pause_timer(state).await?,
            UnpauseTimer => timer_logic::unpause_timer(state).await?,
            StopTimer => timer_logic::stop_timer(state).await?,

            Multiple(_) => bail!("Recursive messages not supported"),
        };
        state.ws_tx.send(response)?;

        Ok(())
    }
}