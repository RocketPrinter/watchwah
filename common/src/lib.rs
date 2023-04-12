pub mod profile;
pub mod timer;
pub mod ws_common;

use std::net::SocketAddr;
use console_subscriber::ConsoleLayer;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use std::path::{Path, PathBuf};

pub fn register_tracing(addr: &str) {
    tracing_subscriber::registry()
        .with(ConsoleLayer::builder().server_addr(addr.parse::<SocketAddr>().unwrap()).with_default_env().spawn())
        .with(tracing_subscriber::fmt::layer().with_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        ))
        .init();
}

pub fn get_config_path() -> PathBuf {
    Path::new(&std::env::var("HOME").unwrap())
        .join(".config")
        .join("watchwah")
}
