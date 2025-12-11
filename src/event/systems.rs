use bevy::prelude::*;

use crate::{
  config::IndigaugeLogLevel,
  event::resources::{BufferedEvents, EventQueueReceiver},
  session::resources::SessionApiKey,
  utils::BevyIndigauge,
};

pub fn maybe_flush_events(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  if ig.buffered_events.events.len() >= ig.config.batch_size {
    ig.flush_events(&session_key);
  }
}

pub fn flush_events(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  if ig.flush_events(&session_key) == 0 {
    ig.send_heartbeat(&session_key);
  }
}

pub fn handle_queued_events(
  receiver: Res<EventQueueReceiver>,
  mut buffered_events: ResMut<BufferedEvents>,
  log_level: Res<IndigaugeLogLevel>,
) {
  for event in receiver.try_iter() {
    match event.validate() {
      Ok(_) => {
        buffered_events.events.push(event);
      },
      Err(error) => {
        if *log_level <= IndigaugeLogLevel::Error {
          error!(message = "Invalid event", ?error);
        }
      },
    }
  }
}
