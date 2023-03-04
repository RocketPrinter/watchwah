use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    // immutable
    pub profile: Profile,

    // mutable
    pub state: TimerState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimerGoal {
    /// overrides EarlyStopBehaviour::Never
    Endless,
    /// total never changes while left shows the remaining time
    Time { left: Duration, total: Duration },
    Todos(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerState {
    pub period: TimerPeriod,
    pub goal: TimerGoal,
    pub pomodoro: Option<PomodoroState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimerPeriod {
    Running {
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    },
    Paused {
        dur_left: Option<Duration>,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PomodoroState {
    pub current_period: PomodoroPeriod,
    /// small breaks since the last long break
    pub small_breaks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PomodoroPeriod { Work, ShortBreak, LongBreak }