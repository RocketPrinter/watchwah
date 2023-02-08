use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};
use crate::daemon::SState;
use crate::common::config::ServerConfig;
use crate::common::profile::Profile;
use crate::common::ws_common::ServerToClient;


#[instrument(name="config monitor", skip_all)]
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
            match load() {
                Ok(new_conf) => {
                    *state.conf.write().await = new_conf;
                    state.current_profile.write().await.take();
                    // cancel existing token and replace it with a new one
                    let mut token = state.cancel_if_profile_changes.lock().await;
                    token.cancel();
                    *token = CancellationToken::new();
                    state.ws_tx.send(serde_json::to_string(&ServerToClient::RefreshedConfig).unwrap()).ok();
                    info!("Successfully reloaded config");
                },
                Err(e) => error!("Failed to parse config: {e}")
            }
        }
    }

    Ok(())
}

fn get_config_path() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap()).join(".config").join("watchwah")
}

pub fn load() ->  Result<ServerConfig> {
    let mut conf: Option<ServerConfig> = None;
    let mut profiles: Vec<Profile> = vec![];

    for file in fs::read_dir(get_config_path())?.filter_map(|f|f.ok()) {
        let path = file.path();
        let Some(ext) = path.extension() else {continue};
        if file.file_name()  == "client.toml" || ext != "toml" {continue}

        let contents = fs::read_to_string(path)?;

        if file.file_name() == "config.toml" {
            // main config
            conf = Some(toml::from_str(&contents)?);
        } else {
            let mut profile = toml::from_str::<Profile>(&contents)?;
            profile.name = file.path().file_stem().unwrap().to_str().unwrap().to_string();
            profiles.push(profile);
        }
    }

    let mut conf = conf.ok_or(anyhow!("config.toml missing!"))?;

    conf.profiles = profiles;
    Ok(conf)
}