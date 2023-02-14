use std::ops::{Deref, DerefMut};
use crate::common::timer_state::{TimerGoal};
use crate::common::ws_common::ServerToClient;
use crate::daemon::SState;
use anyhow::{anyhow, bail, Error, Result};
use tokio::sync::{MutexGuard, RwLockReadGuard};
use crate::common::profile::Profile;
use crate::common::timer_state::CreatedState::Uninit;
use crate::common::timer_state::TimerState::{Created, NotCreated};

// todo: NOT HANDLING AN INVALID PROFILE



pub async fn create_timer(state: &SState, goal: TimerGoal) -> Result<ServerToClient> {
    {
        let mut timer = state.timer.lock().await;
        if let Created {..} = timer.deref() { bail!("Timer is already created!")};
        *timer = Created {
            state: Uninit,
            goal,
            pomodoro: None,
        }
    }
    unpause_timer(state).await
}

pub async fn pause_timer(state: &SState) -> Result<ServerToClient> {
    match state.timer.lock().await.deref_mut() {
        Created { ref mut state, .. } => {
            todo!()
        },
        NotCreated => bail!("Timer is not created!"),
    }
}

pub async fn unpause_timer(state: &SState) -> Result<ServerToClient> {
    match state.timer.lock().await.deref_mut() {
        Created { ref mut state, .. } => {
            todo!()
        },
        NotCreated => bail!("Timer is not created!"),
    }
}

pub async fn stop_timer(state: &SState) -> Result<ServerToClient> {
    let mut timer = state.timer.lock().await;
    if let NotCreated = timer.deref() { bail!("Timer is not created!") }
    state.cancel_timer_tasks.notify_waiters();
    *timer = NotCreated;

    Ok(timer_state_msg(state).await)
}

pub async fn timer_state_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateTimer(state.timer.lock().await.clone())
}