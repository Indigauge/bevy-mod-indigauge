use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
  resources::{events::BufferedEvents, session::SessionApiKey},
  systems::events::{flush_events, handle_queued_events, maybe_flush_events},
};

pub struct EventsPlugin {
  flush_interval: Duration,
}

impl EventsPlugin {
  pub fn new(flush_interval: Duration) -> Self {
    Self { flush_interval }
  }
}

impl Plugin for EventsPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        handle_queued_events,
        maybe_flush_events.run_if(resource_changed::<BufferedEvents>),
        flush_events.run_if(on_timer(self.flush_interval)),
      )
        .run_if(resource_exists::<SessionApiKey>),
    );
  }
}
