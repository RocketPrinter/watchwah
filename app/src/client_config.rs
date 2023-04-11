use std::fs;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use common::config::get_config_path;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClientConfig {
    // todo: key
}

pub fn load_config() -> ClientConfig {
    match fs::read_to_string(get_config_path().join("client.json")) {
        Ok(file) => {
            serde_json::from_str(&file).map_err(|e| error!("Failed to parse config: {e}")).unwrap()
        }
        Err(err) => {
            warn!("Unable to read config: {err}");
            ClientConfig::default()
        }
    }
}