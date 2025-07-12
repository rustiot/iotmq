pub mod cmd;

mod api;
mod config;
mod context;
mod log;
mod mqtt;
mod plugins;
mod protocol;
mod server;
mod session;
mod stream;
mod web;

use config::{Config, Listener as ListenerConfig, Protocol, CFG};
use context::Context;
use log::Log;
use mqtt::MqttServer;
use server::Server;
use session::Session;
use stream::Stream;
use web::{Web, WebServer};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}
