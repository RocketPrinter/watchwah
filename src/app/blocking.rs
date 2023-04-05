use std::path::PathBuf;
use crate::app::{SState, State};
use chrono::Duration;
use std::thread;
use tokio::task::spawn_blocking;
use x11rb::protocol::xproto::Window;

mod x11;

#[derive(Debug)]
pub struct WindowInfo {
    window: Window,
    name: String,
    pid: u32,
    path: Option<PathBuf>,
}

pub async fn blocker_loop(state: SState) {
    let timer_updated = { state.lock().unwrap().timer_updated.clone() };

    loop {
        // we wait until the timer changes state
        timer_updated.notified().await;

        // we can check if we should start the blocker
        if !should_block(&state.lock().unwrap()) {continue;}

        // start blocking
        if cfg!(target_os = "linux") {
            let state = state.clone();
            x11::blocker(state).await;

            // todo: wayland support
        } else {
            // todo: windows support
        }
    }
}

fn should_block(state: &State) -> bool {
    state.ws_connected && state
            .timer
            .as_ref()
            .map(|t| t.state.should_block())
            .unwrap_or(false)
}