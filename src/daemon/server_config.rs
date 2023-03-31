use crate::common::config::{get_config_path};
use crate::common::profile::Profile;
use crate::common::ws_common::ServerToClient;
use crate::daemon::{timer_logic, SState};
use anyhow::{anyhow, bail, Result};
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::unbounded_channel;
use tracing::{error, instrument};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {

    // todo: key

    #[serde(skip)] // generated from neighboring files
    pub profiles: Vec<Profile>,
}

#[instrument(name = "config monitor", skip_all)]
pub async fn config_monitor(state: SState) -> Result<()> {
    let (tx, mut rx) = unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).ok();
    })?;

    watcher.watch(&get_config_path(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        if let EventKind::Create(CreateKind::File)
        | EventKind::Modify(_)
        | EventKind::Remove(RemoveKind::File) = res?.kind
        {
            match tokio::task::spawn_blocking(load_config).await.unwrap() {
                Ok(new_conf) => {
                    *state.conf.write().await = new_conf;
                    if let Ok(msg) = timer_logic::stop_timer(&state).await {
                        state.ws_tx.send(msg).ok();
                    }
                }
                Err(e) => error!("Failed to parse config: {e}"),
            }
        }
    }

    Ok(())
}

pub async fn profiles_msg(state: &SState) -> ServerToClient {
    ServerToClient::UpdateProfiles(
        state
            .conf
            .read()
            .await
            .profiles
            .iter()
            .map(|p| p.name.to_string())
            .collect(),
    )
}

pub fn load_config() -> Result<ServerConfig> {
    let mut conf: Option<ServerConfig> = None;

    let path = get_config_path();
    let config_path = path.join("config.toml");

    if config_path.exists() {
        let contents = fs::read_to_string(config_path)?;
        conf = Some(toml::from_str(&contents)?);
    } else {
        bail!("config.toml missing!");
    }

    let mut conf = conf.ok_or(anyhow!("config.toml missing!"))?;
    conf.profiles = load_profiles(path.join("profiles"))?;
    Ok(conf)
}

fn load_profiles(path: PathBuf) -> Result<Vec<Profile>> {
    let mut profiles = vec![];
    for file in fs::read_dir(path)? {
        let file = file?;
        let path = file.path();
        if path.extension() != Some("toml".as_ref()) {
            continue;
        }

        let contents = fs::read_to_string(path)?;

        let mut profile = toml::from_str::<Profile>(&contents)?;
        profile.name = file
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        profiles.push(profile);
    }

    Ok(profiles)
}
