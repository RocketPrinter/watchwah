use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(skip, default)] // generated from the file name
    pub name: String,

    pub pomodoro: Option<PomodoroSettings>,
    #[serde(default)]
    pub blocking: Blocking,
    #[serde(default = "can_stop_before_goal_is_fulfilled_default")]
    pub can_stop_before_goal_is_fulfilled: bool,
    #[serde(default = "can_pause_default")]
    pub can_pause: bool,
    #[serde(default)]
    pub can_skip_work: bool,
}
fn can_stop_before_goal_is_fulfilled_default() -> bool { true }
fn can_pause_default() -> bool { true }

// todo: use a better format than DurationSeconds
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PomodoroSettings {
    #[serde(default = "work_dur_default")]
    #[serde_as(as = "DurationSeconds<i64>")]
    pub work_dur: Duration,
    #[serde(default = "small_breaks_default")]
    #[serde_as(as = "DurationSeconds<i64>")]
    pub short_break_dur: Duration,
    #[serde(default = "long_breaks_default")]
    #[serde_as(as = "DurationSeconds<i64>")]
    pub long_break_dur: Duration,
    #[serde(default = "break_ratio_default")]
    pub small_breaks_before_big_one: u32,
}
fn work_dur_default() -> Duration { Duration::minutes(25) }
fn small_breaks_default() -> Duration { Duration::minutes(5) }
fn long_breaks_default() -> Duration { Duration::minutes(15) }
fn break_ratio_default() -> u32 { 4 }

impl PomodoroSettings {
    /// calculates the total time the session wil last with breaks included
    pub fn calc_break_time(&self, work_time: Duration) -> Duration {
        let break_periods = (work_time.num_seconds() as f32 / self.work_dur.num_seconds() as f32).ceil() as i32 - 1;
        let long_breaks = break_periods / (self.small_breaks_before_big_one as i32 + 1);
        self.short_break_dur * (break_periods - long_breaks) + self.long_break_dur * long_breaks
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Blocking {
    // regex
    #[serde(default)]
    pub window_names: Vec<String>,
    // path pattern
    #[serde(default)]
    pub process_path: Vec<String>,
    #[serde(default)]
    pub websites: Vec<String>,
    #[serde(default)]
    pub hide_web_video: bool,
}