use bevy::prelude::*;

#[derive(Event, Debug)]
pub enum IndigaugeInitDoneEvent {
  Success,
  Skipped(String),
  Failure(String),
  UnexpectedFailure(String),
}

#[derive(Event, Default, Clone)]
pub struct StartSessionEvent {
  pub platform: Option<String>,
}

impl StartSessionEvent {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
    self.platform = Some(platform.into());
    self
  }
}
