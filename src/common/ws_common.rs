use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;
use crate::common::timer_state::TimerState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    SetActiveProfile(Option<String>),

    PauseTimer,
    StopTimer,
    
    Multiple(Vec<ClientToServer>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    UpdateProfiles(Vec<String>),
    UpdateActiveProfile(Option<Profile>),
    UpdateTimer(TimerState),

    RefreshedConfig,

    Multiple(Vec<ServerToClient>),
}

// todo: todo system