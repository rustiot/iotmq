use crate::MqttServer;
use crate::WebServer;
use daemonize::Daemonize;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs;
use std::io::Write;
use std::process::exit;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};

const PID_FILE: &str = concat!("/tmp/", env!("CARGO_PKG_NAME"), ".pid");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
}

pub struct Server;

impl Server {
    // start server
    pub fn start() {
        // daemon start
        if !cfg!(debug_assertions) {
            if let Ok(pid) = fs::read_to_string(PID_FILE) {
                if let Ok(pid) = pid.trim().parse::<i32>() {
                    if kill(Pid::from_raw(pid), None).is_ok() {
                        println!(
                            "{} server is already running [PID: {}]",
                            env!("CARGO_PKG_NAME"),
                            pid
                        );
                        return;
                    } else {
                        let _ = fs::remove_file(PID_FILE);
                    }
                }
            }

            let daemon =
                Daemonize::new().pid_file(PID_FILE).chown_pid_file(true).working_directory("./");
            if let Err(e) = daemon.start() {
                error!("Daemonize: {}", e);
                exit(1);
            }
        }

        // tokio runtime start
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run());
    }

    // stop server
    pub fn stop() {
        match fs::read_to_string(PID_FILE) {
            Ok(pid) => {
                if let Ok(pid) = pid.trim().parse::<i32>() {
                    if let Err(e) = kill(Pid::from_raw(pid), Signal::SIGTERM) {
                        println!("{} [PID: {}]", e, pid);
                        let _ = fs::remove_file(PID_FILE);
                    }
                }
            }
            Err(_) => {
                println!("{} server is not running", env!("CARGO_PKG_NAME"));
            }
        }
    }

    // restart server
    pub fn restart() {
        info!("{} server restarting...", env!("CARGO_PKG_NAME"));
        Self::stop();
        std::thread::sleep(std::time::Duration::from_secs(1));
        Self::start();
    }

    // reload configuration
    pub fn reload() {}

    // show server status
    pub fn status() {}
}

// run server
async fn run() {
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Web Server
    let rx = shutdown_tx.subscribe();
    let web_task = tokio::spawn(async move {
        if let Err(e) = WebServer::start(rx).await {
            error!("WebServer: {}", e);
            exit(1);
        }
    });

    // Mqtt Server
    let rx = shutdown_tx.subscribe();
    let mqtt_task = tokio::spawn(async move {
        if let Err(e) = MqttServer::start(rx).await {
            error!("MqttServer: {}", e);
            exit(1);
        }
    });

    // Signal listen
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sighup = signal(SignalKind::hangup()).unwrap();
    loop {
        tokio::select! {
            _ = sigint.recv() => {
                info!("Server received SIGINT signal");
                let  _ = shutdown_tx.send(());
                break;
            }
            _ = sigterm.recv() => {
                info!("Server received SIGTERM signal");
                let  _ = shutdown_tx.send(());
                break;
            }
            _ = sighup.recv() => {
                info!("Server received SIGHUP signal");
                Server::reload();
            }
        }
    }

    let _ = tokio::join!(web_task, mqtt_task);

    let _ = fs::remove_file(PID_FILE);
}
