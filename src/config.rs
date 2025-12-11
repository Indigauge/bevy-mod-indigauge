use std::{env, time::Duration};

use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct IndigaugeConfig {
  pub(crate) api_base: String,
  pub(crate) game_name: String,
  pub(crate) public_key: String,
  pub(crate) game_version: String,
  pub(crate) batch_size: usize,
  pub(crate) flush_interval: Duration,
  pub(crate) max_queue: usize,
  pub(crate) request_timeout: Duration,
}

impl IndigaugeConfig {
  pub fn new(game_name: impl Into<String>, public_key: impl Into<String>, game_version: impl Into<String>) -> Self {
    Self {
      api_base: env::var("INDIGAUGE_API_BASE").unwrap_or_else(|_| "https://ingest.indigauge.com".into()),
      game_name: game_name.into(),
      public_key: public_key.into(),
      game_version: game_version.into(),
      batch_size: 64,
      flush_interval: Duration::from_secs(10),
      max_queue: 10_000,
      request_timeout: Duration::from_secs(10),
    }
  }
}

#[derive(Resource, PartialEq, PartialOrd, Clone)]
pub enum IndigaugeLogLevel {
  #[cfg(feature = "tracing")]
  Trace,
  Debug,
  Info,
  Warn,
  Error,
  Silent,
}

#[cfg(feature = "tracing")]
impl IndigaugeLogLevel {
  pub fn as_str(&self) -> &'static str {
    match self {
      IndigaugeLogLevel::Trace => "trace",
      IndigaugeLogLevel::Debug => "debug",
      IndigaugeLogLevel::Info => "info",
      IndigaugeLogLevel::Warn => "warn",
      IndigaugeLogLevel::Error => "error",
      IndigaugeLogLevel::Silent => "silent",
    }
  }
}

#[cfg(feature = "tracing")]
use bevy::utils::tracing::Level;

#[cfg(feature = "tracing")]
impl From<&Level> for IndigaugeLogLevel {
  fn from(level: &Level) -> Self {
    match *level {
      Level::ERROR => IndigaugeLogLevel::Error,
      Level::WARN => IndigaugeLogLevel::Warn,
      Level::INFO => IndigaugeLogLevel::Info,
      Level::DEBUG => IndigaugeLogLevel::Debug,
      Level::TRACE => IndigaugeLogLevel::Trace,
    }
  }
}

#[derive(Resource, PartialEq, Default, Clone)]
pub enum IndigaugeMode {
  /// Live mode sends data to the Indigauge API.
  #[default]
  Live,
  /// Dev mode only logs data to the console (if logging is enabled).
  Dev,
  /// Disabled mode does not send any data to the Indigauge API.
  Disabled,
}
