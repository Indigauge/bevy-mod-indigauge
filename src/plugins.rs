use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_mod_reqwest::ReqwestPlugin;
use crossbeam_channel::bounded;
use serde::Serialize;

use crate::{
  GLOBAL_TX,
  plugins::{events::EventsPlugin, feedback::FeedbackUiPlugin, session::SessionPlugin},
  resources::{
    IndigaugeConfig, IndigaugeLogLevel, IndigaugeMode, LastSentRequestInstant,
    events::{BufferedEvents, EventQueueReceiver, QueuedEvent},
    session::EmptySessionMeta,
  },
};

mod events;
mod feedback;
mod session;

pub struct IndigaugePlugin<Meta = EmptySessionMeta> {
  public_key: String,
  /// Defaults to cargo package name
  game_name: String,
  game_version: String,
  log_level: IndigaugeLogLevel,
  mode: IndigaugeMode,
  meta: PhantomData<Meta>,
}

impl<M> IndigaugePlugin<M> {
  pub fn log_level(mut self, log_level: IndigaugeLogLevel) -> Self {
    self.log_level = log_level;
    self
  }

  pub fn mode(mut self, mode: IndigaugeMode) -> Self {
    self.mode = mode;
    self
  }
}

impl<M> IndigaugePlugin<M>
where
  M: Resource + Serialize,
{
  pub fn new(public_key: impl Into<String>, game_name: Option<String>, game_version: Option<String>) -> Self {
    Self {
      public_key: public_key.into(),
      game_name: game_name.unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string()),
      game_version: game_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    }
  }
}

impl<M> Default for IndigaugePlugin<M>
where
  M: Resource + Serialize,
{
  fn default() -> Self {
    Self {
      game_name: env!("CARGO_PKG_NAME").to_string(),
      public_key: std::env::var("INDIGAUGE_PUBLIC_KEY").unwrap_or_else(|_| {
        warn!("INDIGAUGE_PUBLIC_KEY environment variable not set");
        "".to_string()
      }),
      game_version: env!("CARGO_PKG_VERSION").to_string(),
      log_level: IndigaugeLogLevel::Info,
      mode: IndigaugeMode::default(),
      meta: PhantomData,
    }
  }
}

impl<M> Plugin for IndigaugePlugin<M>
where
  M: Resource + Serialize,
{
  fn build(&self, app: &mut App) {
    let config = IndigaugeConfig::new(&self.game_name, &self.public_key, &self.game_version);

    if matches!(self.mode, IndigaugeMode::Live | IndigaugeMode::Dev) {
      if config.public_key.is_empty() {
        if self.log_level <= IndigaugeLogLevel::Warn {
          warn!("Indigauge public key is not set");
        }
      } else if GLOBAL_TX.get().is_none() {
        let (tx, rx) = bounded::<QueuedEvent>(config.max_queue);
        GLOBAL_TX.set(tx).ok();

        app.insert_resource(EventQueueReceiver::new(rx));
      }
    }

    app
      .add_plugins(ReqwestPlugin::default())
      .add_plugins((
        FeedbackUiPlugin,
        EventsPlugin::new(config.flush_interval),
        SessionPlugin::<M>::new(config.flush_interval),
      ))
      .insert_resource(self.log_level.clone())
      .insert_resource(BufferedEvents::default())
      .insert_resource(LastSentRequestInstant::new())
      .insert_resource(self.mode.clone())
      .insert_resource(config);
  }
}
