use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DurationSeconds;

use crate::timer::{Timer, TimerGoal, TimerState};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[must_use]
pub enum ClientToServer {
    CreateTimer {
        goal: TimerGoal,
        profile_name: String,
        #[serde_as(as = "Option<DurationSeconds<i64>>")]
        start_in: Option<Duration>,
    },
    PauseTimer,
    UnpauseTimer,
    StopTimer,
    SkipPeriod,

    // todo: SetTodos,
    Multiple(Vec<ClientToServer>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[must_use]
pub enum ServerToClient {
    /// (name, pomodoro)
    UpdateProfiles(Vec<ProfileInfo>),
    UpdateTimer(Option<Box<Timer>>),
    UpdateTimerState(Box<TimerState>),

    // todo: UpdateTodos
    RefreshedConfig,

    Multiple(Vec<ServerToClient>),
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileInfo {
    pub name: String,
    #[serde_as(as = "Option<DurationSeconds<i64>>")]
    pub pomo_work_dur: Option<Duration>
}