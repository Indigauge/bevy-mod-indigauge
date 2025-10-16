use bevy::prelude::*;
use std::ops::Deref;

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
