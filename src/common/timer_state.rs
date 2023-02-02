use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimerState {
    Disabled,
    Once {
        dur: Duration,
    },
    Pomodoro {
        period_dur: Duration,
        total_dur: Duration,
        period: Period,
        small_breaks: u16,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Period { Work, ShortBreak, LongBreak }