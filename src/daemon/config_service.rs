use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{error, info, instrument};
use crate::common::config::Config;
use crate::common::profile::Profile;
use crate::common::ws_common::ServerToClient;
use crate::daemon::{SConfig};

#[instrument(name="config monitor", skip_all)]
pub async fn config_monitor(conf: SConfig, ws_tx: Sender<ServerToClient>) -> Result<()> {
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
                    *conf.write().await = new_conf;
                    info!("Successfully reloaded config");
                    ws_tx.send(ServerToClient::RefreshedConfig).ok();
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

pub fn load() ->  Result<Config> {
    let mut conf: Option<Config> = None;
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