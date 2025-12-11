use bevy::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::{
  prelude::StartSessionEvent,
  session::resources::{SessionApiKey, SessionMeta},
  utils::BevyIndigauge,
};

pub fn handle_exit_event<E>(exit_events: EventReader<E>, ig: BevyIndigauge, session_key: Res<SessionApiKey>)
where
  E: Event + std::fmt::Debug,
{
  if !exit_events.is_empty() {
    end_session(ig, session_key);
  }
}

/// System to start a default session.
///
/// This is just a helper to start a default session. And will internally trigger the `StartSessionEvent`, like shown below.
/// If you need to initialize the session with a custom platform, please create your own system to initialize the session.
///
/// ```rust,ignore
/// commands.trigger(StartSessionEvent::default());
/// ```
///
/// # Example Usage:
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_mod_indigauge::prelude::*;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
/// enum GameState {
///   #[default]
///   InitializeSession,
///   Playing
/// }
///
/// fn main() {
///   App::new()
///     .add_plugins(DefaultPlugins)
///     .insert_state(GameState::default())
///     .add_plugins(IndigaugePlugin::<EmptySessionMeta>::default().mode(IndigaugeMode::Dev))
///     .add_systems(OnEnter(GameState::InitializeSession), start_default_session)
///     .add_observer(switch_state_after_session_init(GameState::Playing))
///     .run();
/// }
/// ```
pub fn start_default_session(mut commands: Commands) {
  commands.trigger(StartSessionEvent::default());
}

/// System to end a session. Can be used to end a session manually.
///
/// # Example usage:
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use bevy_mod_indigauge::prelude::*;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
/// enum GameState {
///   #[default]
///   InitializeSession,
///   Playing,
///   Ended
/// }
///
/// fn main() {
///   App::new()
///     .add_plugins(DefaultPlugins)
///     .insert_state(GameState::default())
///     .add_plugins(IndigaugePlugin::<EmptySessionMeta>::default().mode(IndigaugeMode::Dev))
///     .add_systems(OnEnter(GameState::InitializeSession), start_default_session)
///     .add_systems(OnEnter(GameState::Ended), end_session)
///     .add_observer(switch_state_after_session_init(GameState::Playing))
///     .run();
/// }
/// ```
pub fn end_session(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  ig.flush_events(&session_key);

  let reqwest_client = ig.build_post_request("sessions/end", &session_key, &json!({"reason": "ended"}));

  if let Ok(reqwest_client) = reqwest_client {
    ig.reqwest_client.send(reqwest_client);
  }
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
