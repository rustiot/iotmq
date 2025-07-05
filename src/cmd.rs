use crate::Server;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cmd {
    #[command(subcommand)]
    command: Option<SubCmd>,
    #[clap(short, long, value_name = "FILE", global = true)]
    config: Option<String>,
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
    if let Some(config) = cmd.config {
        std::env::set_var("IOTMQ__CONFIG", config);
    }
    match cmd.command.unwrap_or(SubCmd::Start) {
        SubCmd::Start => Server::start(),
        SubCmd::Stop => Server::stop(),
        SubCmd::Restart => Server::restart(),
        SubCmd::Reload => Server::reload(),
        SubCmd::Status => Server::status(),
    }
}
