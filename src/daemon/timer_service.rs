use crate::common::timer_state::TimerState;
use crate::common::ws_common::ServerToClient;
use crate::daemon::SState;
use anyhow::Result;

// todo: more stuff here

pub async fn stop_timer(state: &SState) -> Result<ServerToClient> {
    state.cancel_timer_tasks.notify_waiters();
    *state.timer.lock().await = TimerState::NotCreated;
    
    Ok(timer_state_msg(state).await)
}

pub async fn timer_state_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateTimer(state.timer.lock().await.clone())
}