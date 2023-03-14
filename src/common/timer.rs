use chrono::{Duration, DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use crate::common::profile::Profile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    // immutable
    pub profile: Profile,
    pub goal: TimerGoal,

    // mutable
    pub state: TimerState,
}


#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimerGoal {
    /// overrides EarlyStopBehaviour::Never
    None,
    Time (#[serde_as(as = "DurationSeconds<i64>")]Duration),
    Todos(u32),
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerState {
    /// Doesn't include the current running period
    #[serde_as(as = "DurationSeconds<i64>")]
    pub total_dur: Duration,
    pub period: TimerPeriod,
    pub pomodoro: Option<PomodoroState>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimerPeriod {
    Running {
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    },
    Paused {
        #[serde_as(as = "Option<DurationSeconds<i64>>")]
        total: Option<Duration>,
        #[serde_as(as = "Option<DurationSeconds<i64>>")]
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