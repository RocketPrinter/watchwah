use crate::common::timer::{TimerGoal};
use crate::common::ws_common::ServerToClient;
use crate::daemon::SState;
use anyhow::{bail, Result};

pub async fn create_timer(state: &SState, goal: TimerGoal, profile_name: String) -> Result<ServerToClient> {
    let timer = state.timer.lock().await;
    if timer.is_some() {
        bail!("")
    }


    todo!()
}

pub async fn pause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };



    todo!()
}

pub async fn unpause_timer(state: &SState) -> Result<ServerToClient> {
    let Some(ref mut timer) = *state.timer.lock().await else {
        bail!("Timer is not created")
    };



    todo!()
}

pub async fn stop_timer(state: &SState) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if timer.is_none() { bail!("Timer isn't created!") }
    state.cancel_timer_tasks.notify_waiters();
    *timer = None;

    Ok(timer_state_msg(state).await)
}

pub async fn timer_state_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateTimer(state.timer.lock().await.clone())
}