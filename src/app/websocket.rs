use std::time::Duration;
use clap::builder::Str;
use tokio::{select};
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::time::sleep;
use tracing::{error, event, info, instrument, Level, span};
use websockets::{Frame, WebSocket, WebSocketError};
use crate::app::SState;

const URL: &str = "ws://localhost:63086/";

/// Handles reconnections and message processing
pub async fn ws_loop(state: SState, mut rx: UnboundedReceiver<String>) {
    loop {
        let err = match WebSocket::connect(URL).await {
            Ok(ws) => {
                info!("[Client] Connection established!");
                select_loop(&state, ws,&mut rx).await.unwrap_err()
            },
            Err(err) => err,
        };
        let err = err.to_string();
        error!(err);

        // wait 3 seconds before retrying
        sleep(Duration::from_secs(3)).await;
    }
}

async fn select_loop(state: &SState, mut ws: WebSocket, rx: &mut UnboundedReceiver<String>) -> Result<(), WebSocketError> {
    let mut incomplete_payload: Option<String> = None;
    loop {
        select! {
            // message was received from the websocket that needs to be handled
            received = ws.receive() => {
                if let Frame::Text { payload: frame_payload, fin, .. } = received? {

                    let payload = if let Some(p) = incomplete_payload.take() { p + &frame_payload } else {frame_payload};

                    if fin {
                        // message has finalized, we can handle it
                        handle_msg(state, payload);
                    } else {
                        // message is not finalized yet
                        incomplete_payload = Some(payload);
                    }
                }
            },
            // text was received from channel that needs to be sent to the ws
            Some(text) = rx.recv() => {
                ws.send(Frame::text(text)).await?;
            },
        }
    }
}

fn handle_msg(state: &SState, text: String) {
    todo!()
}