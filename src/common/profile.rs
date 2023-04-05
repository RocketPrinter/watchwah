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
   #[serde(default)]
    pub early_stop: EarlyStopBehaviour,
}

// todo: use a better format than DurationSeconds
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PomodoroSettings {
    #[serde(default = "work_dur_default")]
    #[serde_as(as = "DurationSeconds<i64>")]
    pub work_dur: Duration,
    #[serde(default = "small_breaks_default")]
    #[serde_as(as = "DurationSeconds<i64>")]
    pub small_break_dur: Duration,
    #[serde(default = "long_breaks_default")]
    #[serde_as(as = "DurationSeconds<i64>")]
    pub long_break_dur: Duration,
    #[serde(default = "break_ratio_default")]
    pub small_breaks_between_big_ones: u32,
}
fn work_dur_default() -> Duration { Duration::minutes(25) }
fn small_breaks_default() -> Duration { Duration::minutes(5) }
fn long_breaks_default() -> Duration { Duration::minutes(15) }
fn break_ratio_default() -> u32 { 4 }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Blocking {
    // it's a regex!
    #[serde(default)]
    pub process_name: Vec<String>,
    #[serde(default)]
    pub process_path: Vec<String>,
    #[serde(default)]
    pub websites: Vec<String>,
    #[serde(default)]
    pub hide_web_video: bool,
}

#[serde_as]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum EarlyStopBehaviour {
    #[default]
    NeverAllowed,
    AllowedAfter(#[serde_as(as = "DurationSeconds<i64>")] Duration),
    AlwaysAllowed,
}
