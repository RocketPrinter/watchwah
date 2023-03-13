extern crate core;

mod app;
mod daemon;
pub mod common {
    pub mod config;
    pub mod profile;
    pub mod timer;
    pub mod ws_common;
}

use crate::app::app;
use crate::daemon::daemon;
use axum::handler::Handler;
use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use Command::*;
use DaemonCommand::*;

// todo: more info and args
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the app and the daemon process if necessary
    App,
    /// Commands regarding the daemon
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Start the app and daemon in the same process
    Together,
    // Installs/Updates the firefox addon
    Addon,
    /// Copies the default config to $HOME/config/watchwah. Doesn't replace or modify existing files.
    DefaultConfig,
}

#[derive(Subcommand)]
enum DaemonCommand {
    /// Starts daemon process
    Start,
    /// Kills daemon process
    Kill,
    /// Checks the status of the daemon process
    Status,
    /// Checks the logs of the daemon process
    Logs,
}

fn main() {
    let tokio = init();
    let _guard = tokio.enter();

    match Cli::parse().command {
        App => {
            todo!()
        }
        Daemon { command } => match command {
            Start => {
                todo!()
            }
            Kill => {
                todo!()
            }
            Status => {
                todo!()
            }
            Logs => {
                todo!()
            }
        },
        Together => {
            daemon();
            app();
        }
        Addon => {
            todo!()
        }
        DefaultConfig => {
            todo!()
        }
    }
}

fn init() -> Runtime {
    // logging
    // todo: https://tokio.rs/tokio/topics/tracing-next-steps

    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(tracing_subscriber::fmt::layer().with_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        ))
        .init();

    // tokio runtime
    Runtime::new().unwrap()
}
