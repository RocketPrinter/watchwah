extern crate core;

mod app;
mod daemon;
pub mod common {
    pub mod ws_common;
    pub mod profile;
    pub mod config;
    pub mod timer_state;
}

use clap::{Parser, Subcommand};
use tokio::runtime::{Runtime};
use crate::app::app;
use crate::daemon::daemon;
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
        command: DaemonCommand
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
        }
        Together => {
            tokio::spawn(daemon());
            app();
        },
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
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // tokio runtime
    Runtime::new().unwrap()
}