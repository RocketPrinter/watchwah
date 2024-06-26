use crate::{timer_logic, SState};
use anyhow::{anyhow, bail, Result};
use common::get_config_path;
use common::profile::Profile;
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{error, info, instrument};
use common::ws_common::{ProfileInfo, ServerToClient};

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

    watcher.watch(&get_config_path(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        if let EventKind::Create(CreateKind::File)
        | EventKind::Modify(_)
        | EventKind::Remove(RemoveKind::File) = res?.kind
        {
            match tokio::task::spawn_blocking(load_config).await.unwrap() {
                Ok(new_conf) => {
                    info!("Config updated!");
                    {
                        *state.conf.write().await = new_conf;
                    }

                    let mut timer = state.timer.lock().await;
                    if timer.is_some() {
                        // we don't propagate the error as that would stop the monitor
                        match timer_logic::stop_timer(&mut *timer, &state) {
                            Ok(msg) => {
                                let msg = ServerToClient::Multiple(vec![
                                    profiles_msg(state.conf.read().await.deref()),
                                    msg.to_msg(timer.as_ref()).unwrap(),
                                ]);
                                state.ws_tx.send(msg).ok();
                            }
                            Err(e) => error!("Failed to stop timer: {e}"),
                        }
                    }
                }
                Err(e) => error!("Failed to parse config: {e}"),
            }
        }
    }

    Ok(())
}

pub fn profiles_msg(conf: &ServerConfig) -> ServerToClient {
    ServerToClient::UpdateProfiles(conf.profiles.iter().map(|p| ProfileInfo{
        name: p.name.to_string(),
        pomodoro: p.pomodoro.clone(),
    }).collect())
}

pub fn load_config() -> Result<ServerConfig> {
    let path = get_config_path();
    let config_path = path.join("config.toml");

    let mut conf: ServerConfig = if config_path.exists() {
        let contents = fs::read_to_string(config_path)?;
        toml::from_str(&contents)?
    } else {
        bail!("config.toml missing!");
    };

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
        profile.name = file.path().file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Config file has no path!"))?
            .to_string();
        profiles.push(profile);
    }

    Ok(profiles)
}
