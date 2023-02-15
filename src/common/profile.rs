use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profile {
    #[serde(skip, default)] // generated from the file name
    pub name: String,

    #[serde(default)]
    pub pomodoro: Option<PomodoroSettings>,
    #[serde(default)]
    pub blocking: Blocking,
    #[serde(default)]
    pub early_stop: EarlyStopBehaviour,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PomodoroSettings {
    pub work_dur: Duration,
    pub small_break_dur: Duration,
    pub long_break_dur: Option<Duration>,
    #[serde(default = "break_ratio_default")]
    /// x short breaks to 1 long break
    pub break_ratio: u32,
}
fn break_ratio_default() -> u32 {3}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Blocking {
    #[serde(default)]
    pub processes: Vec<String>,
    #[serde(default)]
    pub websites: Vec<String>,
    #[serde(default)]
    pub hide_web_video: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EarlyStopBehaviour {
    NeverAllowed,
    AllowedAfter(Duration),
    AlwaysAllowed,
}
impl Default for EarlyStopBehaviour {
    fn default() -> Self {
        EarlyStopBehaviour::AlwaysAllowed
    }
}