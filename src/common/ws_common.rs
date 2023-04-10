use serde::{Deserialize, Serialize};

use crate::common::timer::{TimerGoal, Timer, TimerState};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    CreateTimer { goal: TimerGoal, profile_name: String },
    PauseTimer,
    UnpauseTimer,
    StopTimer,

    // todo: SetTodos,
    
    Multiple(Vec<ClientToServer>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    UpdateProfiles(Vec<String>),
    UpdateTimer(Option<Box<Timer>>),
    UpdateTimerState(TimerState),

    // todo: UpdateTodos

    RefreshedConfig,

    Multiple(Vec<ServerToClient>),
}

impl ClientToServer {
    pub fn chain(self, msg: Self) -> Self {
        if let Self::Multiple(mut msgs) = self {
            msgs.push(msg);
            Self::Multiple(msgs)
        } else {
            Self::Multiple(vec![self, msg])
        }
    }
}

impl ServerToClient {
    pub fn chain(self, msg: Self) -> Self {
        if let Self::Multiple(mut msgs) = self {
            msgs.push(msg);
            Self::Multiple(msgs)
        } else {
            Self::Multiple(vec![self, msg])
        }
    }
}