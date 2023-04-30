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
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimerGoal {
    #[default] None,
    Todos(u32),

    Time(#[serde_as(as = "DurationSeconds<i64>")] Duration),
    Pomodoros(u32),
}

impl TimerGoal {
    /// total time limit
    pub fn time_limit(timer: &Timer) -> Option<Duration> {
        match timer.goal {
            TimerGoal::None | TimerGoal::Todos(_) => None,
            TimerGoal::Time(dur) => Some(dur),
            TimerGoal::Pomodoros(n) => Some(timer.profile.pomodoro.as_ref()?.work_dur * n as i32),
        }
    }

    // time left of the time limit
    pub fn time_left(timer: &Timer) -> Option<Duration> {
        TimerGoal::time_limit(timer).map(|dur| dur - timer.state.total_dur_worked)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerState {
    pub progress: PeriodProgress,
    pub period: PeriodType,

    /// total duration worked, includes current work period
    #[serde_as(as = "DurationSeconds<i64>")]
    pub total_dur_worked: Duration,
    /// small breaks since the last long break
    pub small_breaks: u32,
}

impl TimerState {
    pub fn should_block(&self) -> bool {
        self.progress.is_running() && matches!(self.period, PeriodType::Work)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PeriodProgress {
    Uninit,
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
use PeriodProgress::*;

impl PeriodProgress {
    pub fn is_running(&self) -> bool {
        match self {
            Uninit => false,
            Running { .. } => true,
            Paused { .. } => false,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self {
            Uninit => Duration::zero(),
            Running { elapsed, start, .. } => *elapsed + (Utc::now() - *start),
            Paused { elapsed, .. } => *elapsed,
        }
    }

    pub fn limit(&self) -> Option<Duration> {
        match self {
            Uninit => Some(Duration::zero()),
            Running { limit, .. } => *limit,
            Paused { limit, .. } => *limit,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeriodType {
    Uninit,
    Work,
    StartingBreak,
    ShortBreak,
    LongBreak,
}
