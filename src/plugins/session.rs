use bevy::{prelude::*, window::WindowCloseRequested};

use crate::{
  StartSessionEvent, observers::observe_start_session_event, resources::session::SessionApiKey, systems::session::handle_exit_event,
};

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<StartSessionEvent>()
      .add_observer(observe_start_session_event)
      .add_systems(
        PostUpdate,
        (handle_exit_event::<AppExit>, handle_exit_event::<WindowCloseRequested>)
          .run_if(resource_exists::<SessionApiKey>),
      );
  }
}
