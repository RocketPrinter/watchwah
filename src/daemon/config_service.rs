use crate::common::config;
use anyhow::Result;
use notify::event::{CreateKind, RemoveKind};
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc::unbounded_channel;

pub async fn config_monitor(/* todo: args*/) -> Result<()> {
    let (tx, mut rx) = unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).ok();
    })?;

    watcher.watch(&config::get_config_path(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        // todo: is result handled correctly?
        if let EventKind::Create(CreateKind::File)
        | EventKind::Modify(_)
        | EventKind::Remove(RemoveKind::File) = res?.kind {
            // todo: load config
        }
    }

    Ok(())
}
