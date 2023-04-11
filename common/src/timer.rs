use crate::profile::Profile;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

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
    Time(#[serde_as(as = "DurationSeconds<i64>")] Duration),
    // todo: Pomodoro(u32)
    Todos(u32),
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerState {
    pub period: TimerPeriod,
    pub pomodoro: Option<PomodoroState>,
}

impl TimerState {
    pub fn should_block(&self) -> bool {
        self.period.is_running()
            && self
                .pomodoro.as_ref()
                .map(|p| p.current_period == PomodoroPeriod::Work)
                .unwrap_or(true)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PomodoroState {
    /// total duration of previous work periods
    #[serde_as(as = "DurationSeconds<i64>")]
    pub total_dur_worked: Duration,
    pub current_period: PomodoroPeriod,
    /// small breaks since the last long break
    pub small_breaks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PomodoroPeriod {
    Work,
    ShortBreak,
    LongBreak,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimerPeriod {
    Running {
        // doesn't include the time between start and now
        #[serde_as(as = "DurationSeconds<i64>")]
        elapsed: Duration,
        // time since last pause or start of timer
        start: DateTime<Utc>,
        // duration at which the timer stops
        #[serde_as(as = "Option<DurationSeconds<i64>>")]
        limit: Option<Duration>,
    },
    Paused {
        #[serde_as(as = "DurationSeconds<i64>")]
        elapsed: Duration,
        // duration at which the timer stops
        #[serde_as(as = "Option<DurationSeconds<i64>>")]
        limit: Option<Duration>,
    },
}

impl TimerPeriod {
    pub fn is_running(&self) -> bool {
        match self {
            TimerPeriod::Running { .. } => true,
            TimerPeriod::Paused { .. } => false,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self {
            TimerPeriod::Running { elapsed, start, .. } => *elapsed + (Utc::now() - *start),
            TimerPeriod::Paused { elapsed, .. } => *elapsed,
        }
    }

    pub fn limit(&self) -> Option<Duration> {
        match self {
            TimerPeriod::Running { limit, .. } => *limit,
            TimerPeriod::Paused { limit, .. } => *limit,
        }
    }
}
