use bevy::prelude::*;
use serde_json::json;

use crate::{resources::session::SessionApiKey, sysparam::BevyIndigauge};

pub fn handle_exit_event<E>(mut exit_events: EventReader<E>, mut ig: BevyIndigauge, session_key: Res<SessionApiKey>)
where
  E: Event + std::fmt::Debug,
{
  exit_events.read().for_each(|_event| {
    let reqwest_client = ig.build_request("sessions/end", &session_key, &json!({"reason": "ended"}));

    if let Ok(reqwest_client) = reqwest_client {
      ig.reqwest_client.send(reqwest_client);
    }

    ig.flush_events(&session_key);
  });
}
