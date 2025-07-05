use crate::Log;
use crate::Web;
use config::{Environment, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

// Configuration error
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("config file [{0}] not found")]
    NotFound(String),
    #[error("config deserialize error: {0}")]
    ConfigError(#[from] config::ConfigError),
}

// Configuration struct
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "listener")]
    pub listeners: HashMap<Protocol, Listener>,
    pub log: Log,
    pub web: Web,
}

// Listener configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Listener {
    pub addr: SocketAddr,
    #[serde(default)]
    pub cert: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub proxy_protocol: bool,
    #[serde(default)]
    pub max_connections: usize,
}

// Listener protocol
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Protocol {
    Tcp,
    Tls,
    Ws,
    Wss,
}

impl FromStr for Protocol {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(Protocol::Tcp),
            "tls" => Ok(Protocol::Tls),
            "ws" => Ok(Protocol::Ws),
            "wss" => Ok(Protocol::Wss),
            _ => Err(format!("unknown protocol: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for Protocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Self::from_str(&str).map_err(serde::de::Error::custom)
    }
}

// Global static configuration variable
pub static CFG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| match Config::load() {
    Ok(config) => Arc::new(RwLock::new(config)),
    Err(e) => {
        eprintln!("{}", e);
        exit(1)
    }
});

impl Config {
    fn load() -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder();

        if let Ok(config) = std::env::var("IOTMQ__CONFIG") {
            if std::path::Path::new(&config).exists() {
                builder = builder.add_source(File::with_name(&config));
            } else {
                return Err(ConfigError::NotFound(config));
            }
        } else {
            builder = builder
                .add_source(File::with_name("/etc/iotmq/iotmq").required(false))
                .add_source(File::with_name("config/iotmq").required(false));
        }

        builder =
            builder.add_source(Environment::with_prefix("iotmq").separator("__").try_parsing(true));
        builder.build()?.try_deserialize().map_err(|e| e.into())
    }

    pub fn reload() {}

    pub async fn get() -> Self {
        CFG.read().await.clone()
    }
}
