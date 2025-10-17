use bevy::prelude::*;

#[derive(Event, Default)]
pub struct StartSessionEvent {
  pub locale: Option<String>,
  pub platform: Option<String>,
}

impl StartSessionEvent {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
    self.locale = Some(locale.into());
    self
  }

  pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
    self.platform = Some(platform.into());
    self
  }
}
