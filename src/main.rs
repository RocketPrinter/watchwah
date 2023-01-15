mod daemon;
mod app;

use std::env::args;
use clap::{Parser, Subcommand};

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
    // logging
    // todo: https://tokio.rs/tokio/topics/tracing-next-steps
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // tokio runtime
    let tokio = tokio::runtime::Runtime::new()
        .unwrap();
    let _guard = tokio.enter();

    app::app();

    return;

    match Cli::parse().command {
        Command::App => {
            todo!()
        }
        Command::Daemon { command } => match command {
            DaemonCommand::Start => {
                todo!()
            }
            DaemonCommand::Kill => {
                todo!()
            }
            DaemonCommand::Status => {
                todo!()
            }
            DaemonCommand::Logs => {
                todo!()
            }
        }
        Command::Together => {
            todo!()
        }
    }
}