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

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PomodoroSettings {
    #[serde_as(as = "DurationSeconds<i64>")]
    pub work_dur: Duration,
    #[serde_as(as = "DurationSeconds<i64>")]
    pub small_break_dur: Duration,
    #[serde_as(as = "Option<DurationSeconds<i64>>")]
    pub long_break_dur: Option<Duration>,
    #[serde(default = "break_ratio_default")]
    /// x short breaks to 1 long break
    pub break_ratio: u32,
}
fn break_ratio_default() -> u32 {
    3
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Blocking {
    #[serde(default)]
    pub processes: Vec<String>,
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
