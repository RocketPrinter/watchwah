use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;

pub fn get_config_path() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap())
        .join(".config")
        .join("watchwah")
}