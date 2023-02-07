use tokio::sync::broadcast::Sender;
use crate::common::timer_state::TimerState;

pub struct TimerService {
    state: TimerState,
    ws_tx: Sender<String>,
}

impl TimerService {
    fn new(ws_tx: Sender<String>) -> Self {
        TimerService {
            state: TimerState::NotCreated,
            ws_tx
        }
    } 
}