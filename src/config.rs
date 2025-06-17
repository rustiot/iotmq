use crate::log::Log;
use config::{Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

// Configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub listener: HashMap<String, Listener>,
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Listener {
    pub addr: String,
    #[serde(default)]
    pub cert: String,
    #[serde(default)]
    pub key: String,
}

static CFG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| Arc::new(RwLock::new(Config::load())));

impl Config {
    fn load() -> Self {
        config::Config::builder()
            .add_source(File::with_name("/etc/iotmq/iotmq").required(false))
            .add_source(File::with_name("/etc/iotmq").required(false))
            .add_source(File::with_name("config/iotmq").required(false))
            .add_source(
                Environment::with_prefix("iotmq")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()
            .expect("Failed to build config")
            .try_deserialize()
            .expect("Failed to deserialize config")
    }

    pub fn reload() {}

    pub fn get() -> Self {
        CFG.read().unwrap().clone()
    }
}
