use std::time::Duration;
use tokio::{select};
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::time::sleep;
use tracing::{error, info, instrument};
use websockets::{Frame, WebSocket, WebSocketError};
use anyhow::Result;
use crate::SState;
use common::ws_common::{ClientToServer, ServerToClient};
use ServerToClient::*;

const URL: &str = "ws://127.0.0.1:63086/ws";

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
        if let Ok(mut state) = state.lock(){
            state.ws_connected = false;
            if let Some(ref ctx) = state.egui_context {ctx.request_repaint()}
        }

        // wait 3 seconds before retrying
        sleep(Duration::from_secs(3)).await;
    }
}

async fn select_loop(state: &SState, mut ws: WebSocket, rx: &mut UnboundedReceiver<ClientToServer>) -> Result<(),WebSocketError> {
    let mut incomplete_payload: Option<String> = None;
    if let Ok(mut state) = state.lock() {
        state.ws_connected = true;
        if let Some(ref ctx) = state.egui_context {ctx.request_repaint()}
    }
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
                            Err(e) => error!("Failed to deserialize message: {e}")
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
                    Err(e) => error!("Failed to serialize message: {e}"),
                },
        }
    }
}

fn handle_msg(state: &SState, msg: ServerToClient) {
    if let Multiple(msgs) = msg {
        for msg in msgs { handle_msg(state, msg); }
        return;
    }

    let mut state = state.lock().unwrap();

    match msg {
        UpdateProfiles(profiles) => { state.profiles = profiles; },
        UpdateTimer(timer) => { state.timer = timer; state.timer_updated.notify_one(); }
        UpdateTimerState(timer_state) => if let Some(ref mut timer) = state.timer {
            timer.state = timer_state;
            state.timer_updated.notify_one();
        }

        RefreshedConfig => todo!(), // show a popup

        Multiple(_) => panic!(),
    }

    if let Some(ref ctx) = state.egui_context {ctx.request_repaint()}
}