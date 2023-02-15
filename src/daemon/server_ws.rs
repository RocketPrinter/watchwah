use crate::common::ws_common::{ClientToServer, ServerToClient};
use crate::daemon::{config_service, SState, timer_service};
use anyhow::{bail, Result};
use axum::extract::ws::{Message, WebSocket};
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tracing::{error, info, instrument, warn};
use ClientToServer::*;

#[instrument(name = "server ws", skip_all)]
pub async fn handle_socket(mut ws: WebSocket, state: SState, mut rx: Receiver<ServerToClient>) {
    info!("Connection established"); // todo: log ip/other info2x

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
                    Ok(msg) => if let Err(e) = handle_send(&mut ws, msg).await {
                        error!("Failed to send message: {e}");
                    },
                    Err(e) => error!("Broadcast error: {e}"),
                }}
        }
    }
}

async fn send_welcome_message(ws: &mut WebSocket, state: &SState) -> Result<()> {
    // todo: too many clones :/
    let msg = ServerToClient::Multiple(vec![
        config_service::profiles_msg(state).await,
        timer_service::timer_state_msg(state).await,
    ]);
    handle_send(ws, msg).await?;
    Ok(())
}

async fn handle_send(ws: &mut WebSocket, msg: ServerToClient) -> Result<()> {
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
        match msg {
            CreateTimer { goal, profile_name } => {state.ws_tx.send(timer_service::create_timer(state, goal, profile_name).await?)?;}
            PauseTimer => {state.ws_tx.send(timer_service::pause_timer(state).await?)?;},
            UnpauseTimer => {state.ws_tx.send(timer_service::unpause_timer(state).await?)?;},
            StopTimer => {state.ws_tx.send(timer_service::stop_timer(state).await?)?;},

            Multiple(_) => bail!("Recursive messages not supported"),
        }

        Ok(())
    }
}