use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    #[serde(skip)] // generated from the file name
    pub name: String,

    pub timer: TimerSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimerSettings {
    #[serde(default)]
    stopping: TimerBehaviour,
    pausing: TimerBehaviour,
}

#[derive(Serialize, Deserialize, Debug)]
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