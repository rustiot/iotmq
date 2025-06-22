use crate::context::Context;
use crate::{log, MqttServer, WebServer};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs;
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

            let stdout = fs::File::create("/tmp/daemon.out").unwrap();
            let stderr = fs::File::create("/tmp/daemon.err").unwrap();
            let daemon = daemonize::Daemonize::new()
                .pid_file(PID_FILE)
                .chown_pid_file(true)
                .working_directory("./")
                .stdout(stdout)
                .stderr(stderr);
            if let Err(e) = daemon.start() {
                eprintln!("Daemonize failed: {}", e);
                exit(1);
            }
        }

        // tokio runtime start
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run());
    }

    // stop server
    pub fn stop() {
        if let Ok(pid) = fs::read_to_string(PID_FILE) {
            if let Ok(pid) = pid.trim().parse::<i32>() {
                if let Err(e) = kill(Pid::from_raw(pid), Signal::SIGTERM) {
                    println!("{} [PID: {}]", e, pid);
                    let _ = fs::remove_file(PID_FILE);
                }
            }
        }
    }

    fn is_stop() -> bool {
        !std::path::Path::new(PID_FILE).exists()
    }

    // restart server
    pub fn restart() {
        Self::stop();
        for _ in 0..30 {
            if Self::is_stop() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Self::start();
    }

    // reload configuration
    pub fn reload() {}

    // show server status
    pub fn status() {}
}

// run server
async fn run() {
    log::init();
    let ctx = Context::new().build();

    // Web Server
    let web_ctx = ctx.clone();
    let web_task = tokio::spawn(async move {
        if let Err(e) = WebServer::start(web_ctx).await {
            error!("WebServer: {}", e);
            exit(1);
        }
    });

    // Mqtt Server
    let mqtt_ctx = ctx.clone();
    let mqtt_task = tokio::spawn(async move {
        if let Err(e) = MqttServer::start(mqtt_ctx).await {
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
                info!("server received SIGINT signal");
                let  _ = ctx.shutdown();
                break;
            }
            _ = sigterm.recv() => {
                info!("server received SIGTERM signal");
                let  _ = ctx.shutdown();
                break;
            }
            _ = sighup.recv() => {
                info!("server received SIGHUP signal");
                Server::reload();
            }
        }
    }

    let _ = tokio::join!(web_task, mqtt_task);
    let _ = fs::remove_file(PID_FILE);
}
