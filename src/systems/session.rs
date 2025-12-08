use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::{
  resources::session::{SessionApiKey, SessionMeta},
  sysparam::BevyIndigauge,
};

pub fn handle_exit_event<E>(mut exit_events: EventReader<E>, mut ig: BevyIndigauge, session_key: Res<SessionApiKey>)
where
  E: Event + std::fmt::Debug,
{
  exit_events.read().for_each(|_event| {
    let reqwest_client = ig.build_post_request("sessions/end", &session_key, &json!({"reason": "ended"}));

    if let Ok(reqwest_client) = reqwest_client {
      ig.reqwest_client.send(reqwest_client);
    }

    ig.flush_events(&session_key);
  });
}

pub(crate) fn handle_updated_metadata<M>(mut session_meta: ResMut<SessionMeta<M>>)
where
  M: Resource + Serialize,
{
  session_meta.is_changed = true;
}

pub(crate) fn update_metadata<M>(
  mut session_meta: ResMut<SessionMeta<M>>,
  metadata: Res<M>,
  mut ig: BevyIndigauge,
  session_key: Res<SessionApiKey>,
) where
  M: Resource + Serialize,
{
  if session_meta.is_changed {
    session_meta.is_changed = false;

    ig.update_metadata(&*metadata, &session_key);
  }
}
