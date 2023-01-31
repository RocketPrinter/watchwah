use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(skip)] // generated from the file name
    pub name: String,

    pub timer: TimerConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimerConfig {
    #[serde(default)]
    stopping: TimerBehaviour,
    pausing: TimerBehaviour,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TimerBehaviour{
    Never,
    After(Duration),
    Always,
}

impl Default for TimerBehaviour {
    fn default() -> Self {
        Self::Always
    }
}