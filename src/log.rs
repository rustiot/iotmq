use crate::Config;
use chrono::Local;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::filter::{FilterFn, LevelFilter};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer, Registry};

static LOG_GUARD: OnceCell<(WorkerGuard, WorkerGuard)> = OnceCell::new();

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
        env!("CARGO_PKG_NAME").to_string() + ".log"
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
fn deserialize_level<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Level, D::Error> {
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
    if !LOG_GUARD.get().is_none() {
        return;
    }

    if cfg!(debug_assertions) {
        fmt().with_timer(Timer).with_max_level(Level::DEBUG).init();
    } else {
        let config = Config::get().log;
        let is_json = config.format.to_lowercase() == "json";

        // error log
        let error_file = rolling::never(&config.dir, "error.log");
        let (error_writer, error_guard) = non_blocking(error_file);
        let error_layer = if is_json {
            fmt::layer().json().with_writer(error_writer).with_filter(LevelFilter::ERROR).boxed()
        } else {
            fmt::layer().pretty().with_writer(error_writer).with_filter(LevelFilter::ERROR).boxed()
        };

        // other log
        let log_file = rolling::daily(&config.dir, &config.file);
        let (log_writer, log_guard) = non_blocking(log_file);
        let log_filter = FilterFn::new(move |meta| {
            meta.level() != &Level::ERROR && meta.level() <= &config.level
        });
        let log_layer = if is_json {
            fmt::layer().json().with_writer(log_writer).with_filter(log_filter).boxed()
        } else {
            fmt::layer().pretty().with_writer(log_writer).with_filter(log_filter).boxed()
        };

        Registry::default().with(log_layer).with(error_layer).init();

        LOG_GUARD.set((error_guard, log_guard)).ok();
    }
}
