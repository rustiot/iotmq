use clap::{Parser, Subcommand};
use tokio::time;
use tracing::{debug, error, info, warn};

#[derive(Parser)]
#[command(version, about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cmd {
    #[command(subcommand)]
    command: Option<SubCmd>
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
        SubCmd::Start => start(),
        SubCmd::Stop => stop(),
        SubCmd::Restart => restart(),
        SubCmd::Reload => reload(),
        SubCmd::Status => status(),
    }
}

fn start() {
    info!("start");
    loop {
        std::thread::sleep(time::Duration::from_secs(5));
        debug!("loop");
        info!("loop {}","sdsd");
        warn!("loop");
        error!(user_id = 42, request_id = "abc123", "loop");
    }
}

fn stop() {
    println!("Stop");
}

fn restart() {
    println!("Restart");
}

fn reload() {
    println!("Reload");
}

fn status(){
    println!("Status");
}
