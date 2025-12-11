use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use crossbeam_channel::Receiver;

use crate::api_types::EventPayload;

#[derive(Clone, Debug)]
pub struct QueuedEvent {
  payload: EventPayload,
}

impl QueuedEvent {
  pub fn new(payload: EventPayload) -> Self {
    Self { payload }
  }

  pub fn into_inner(self) -> EventPayload {
    self.payload
  }

  pub fn validate(&self) -> Result<(), String> {
    // Add validation logic here
    let (ns, t) = self.payload.event_type.split_once('.').ok_or("Invalid event type")?;
    if ns.trim().is_empty() || t.trim().is_empty() {
      return Err("Invalid event type".to_string());
    }

    Ok(())
  }
}

#[derive(Resource)]
pub struct EventQueueReceiver {
  rx: Receiver<QueuedEvent>,
}

impl EventQueueReceiver {
  pub fn new(rx: Receiver<QueuedEvent>) -> Self {
    Self { rx }
  }
}

impl Deref for EventQueueReceiver {
  type Target = Receiver<QueuedEvent>;

  fn deref(&self) -> &Self::Target {
    &self.rx
  }
}

impl DerefMut for EventQueueReceiver {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.rx
  }
}

#[derive(Resource, Default)]
pub struct BufferedEvents {
  pub events: Vec<QueuedEvent>,
}
