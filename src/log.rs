use crate::config::Config;
use chrono::Local;
use std::str::FromStr;
use serde::{Deserialize, Deserializer};
use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

// Log configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Log {
    #[serde(deserialize_with = "deserialize_level")]
    pub level: Level,
    pub format: String,
    pub dir: String,
    pub file: String,
}

impl Log {
    #[inline]
    fn default_level() -> Level {
        Level::INFO
    }
    #[inline]
    fn default_format() -> String {
        "json".into()
    }
    #[inline]
    fn default_dir() -> String {
        "./logs".into()
    }
    #[inline]
    fn default_file() -> String {
        env!("CARGO_PKG_NAME").to_string()+ ".log"
    }
}

// Log configuration default
impl Default for Log {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            format: Self::default_format(),
            dir: Self::default_dir(),
            file: Self::default_file(),
        }
    }
}

// Deserialize for level
fn deserialize_level<'de, D:Deserializer<'de>>(deserializer: D) -> Result<Level, D::Error> {
    let level = String::deserialize(deserializer)?;
    let level = Level::from_str(&level).unwrap_or(Level::INFO);
    Ok(level)
}

// Time formatting
struct Timer;

impl FormatTime for Timer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

// Log initialization
pub fn init() {
    let config = Config::get().log;

    if cfg!(debug_assertions) {
        fmt().with_max_level(Level::DEBUG).init();
    } else {
        let error_file =
            rolling::never(&config.dir, "error.log").with_filter(|meta| meta.level() == &Level::ERROR);
        let log_file = rolling::daily(&config.dir, &config.file)
            .with_max_level(config.level)
            .with_filter(|meta| meta.level() != &Level::ERROR);
        let files = error_file.and(log_file);

        let builder = fmt().with_timer(Timer).with_writer(files);

        if config.format == "json" {
            builder.json().init();
        } else {
            builder.pretty().init()
        }
    }
}
