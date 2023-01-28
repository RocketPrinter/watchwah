use crate::common::config;
use anyhow::Result;
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{error, info, instrument};
use crate::common::ws_common::ServerToClient;
use crate::daemon::SConfig;

#[instrument(name="config monitor", skip_all)]
pub async fn config_monitor(conf: SConfig, ws_tx: Sender<ServerToClient>) -> Result<()> {
    let (tx, mut rx) = unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).ok();
    })?;

    watcher.watch(&config::get_config_path(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        if let EventKind::Create(CreateKind::File)
        | EventKind::Modify(_)
        | EventKind::Remove(RemoveKind::File) = res?.kind
        {
            match config::load(&config::get_config_path()) {
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
