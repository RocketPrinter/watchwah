use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::common::profile::Profile;

// todo: move port here

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {

    // todo: key

    #[serde(skip)] // generated from neighboring files
    pub profiles: Vec<Profile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    // todo: key
}

pub fn get_config_path() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap())
        .join(".config")
        .join("watchwah")
}