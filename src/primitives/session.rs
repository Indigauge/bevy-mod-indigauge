use bevy::prelude::*;

#[derive(Event, Default)]
pub struct StartSessionEvent {
  pub locale: Option<String>,
  pub platform: Option<String>,
}
