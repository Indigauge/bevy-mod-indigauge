use std::{
  env,
  ops::Deref,
  time::{Duration, Instant},
};

use bevy::ecs::system::Resource;
use bevy_mod_reqwest::BevyReqwest;

pub mod events;

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

#[derive(Resource)]
pub struct SessionApiKey {
  key: String,
}

impl SessionApiKey {
  pub(crate) fn new(key: impl Into<String>) -> Self {
    Self { key: key.into() }
  }
}

impl Deref for SessionApiKey {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.key
  }
}

#[derive(Resource)]
pub struct LastSentRequestInstant {
  pub(crate) instant: Instant,
}

impl LastSentRequestInstant {
  pub fn new() -> Self {
    Self {
      instant: Instant::now(),
    }
  }
}

#[derive(Resource, PartialEq, PartialOrd, Clone)]
pub enum IndigaugeLogLevel {
  Debug,
  Info,
  Warn,
  Error,
  Silent,
}
