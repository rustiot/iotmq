use crate::Server;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cmd {
    #[command(subcommand)]
    command: Option<SubCmd>,
}

#[derive(Subcommand)]
enum SubCmd {
    /// Start IotMQ
    Start,
    /// Stop IotMQ
    Stop,
    /// Restart IotMQ
    Restart,
    /// Reload configuration file
    Reload,
    /// Show IotMQ Status
    Status,
}

pub fn parse() {
    let cmd = Cmd::parse();
    match cmd.command.unwrap_or(SubCmd::Start) {
        SubCmd::Start => Server::start(),
        SubCmd::Stop => Server::stop(),
        SubCmd::Restart => Server::restart(),
        SubCmd::Reload => Server::reload(),
        SubCmd::Status => Server::status(),
    }
}
