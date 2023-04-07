use std::path::PathBuf;
use crate::app::{SState, State};
use chrono::Duration;
use std::thread;
use tokio::task::spawn_blocking;
use x11rb::protocol::xproto::Window;
use anyhow::Result;
use pathpatterns::{MatchEntry, MatchList, MatchType, Pattern};

mod x11;

pub async fn blocker_loop(state: SState) {
    let timer_updated = { state.lock().unwrap().timer_updated.clone() };

    loop {
        // we wait until the timer changes state
        timer_updated.notified().await;

        // we can check if we should start the blocker
        if !should_enable_blocker(&state.lock().unwrap()) {continue;}

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

fn should_enable_blocker(state: &State) -> bool {
    state.ws_connected && state
            .timer
            .as_ref()
            .map(|t| t.state.should_block())
            .unwrap_or(false)
}

//todo: caching caching caching caching caching
fn should_block_windows(state: &State, process_name: &str, process_path: Option<&str>) -> bool {
    let blocking = &state.timer.as_ref().unwrap().profile.blocking;
    for str in blocking.window_names.iter() {
        // todo: bad bad bad bad bad
        if regex::Regex::new(str).unwrap().is_match(process_name) {
            return true;
        }
    }
    if let Some(process_path) = process_path {
        // todo: bad bad bad bad bad
        let match_list = blocking.process_path.iter().map(|str| MatchEntry::include(Pattern::path(str).unwrap())).collect::<Vec<MatchEntry>>();
        if match_list.matches(process_path, None) == Some(MatchType::Include) {
            return true;
        }
    }

    false
}