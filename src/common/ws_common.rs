use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;
use crate::common::timer_state::{TimerGoal, TimerState};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    SetActiveProfile(Option<String>),

    Create(TimerGoal),
    PauseTimer,
    UnpauseTimer,
    StopTimer,

    // todo: SetTodos,
    
    Multiple(Vec<ClientToServer>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    UpdateProfiles(Vec<String>),
    UpdateActiveProfile(Option<Profile>),
    UpdateTimer(TimerState),

    // todo: UpdateTodos

    RefreshedConfig,

    Multiple(Vec<ServerToClient>),
}