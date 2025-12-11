use bevy::prelude::*;
use bevy_text_edit::TextEditPluginNoState;

use crate::{feedback::resources::*, feedback::systems::*, session::resources::SessionApiKey};

pub mod components;
pub mod helpers;
pub(crate) mod observers;
pub mod resources;
mod systems;
pub mod types;

pub struct FeedbackUiPlugin;

impl Plugin for FeedbackUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(TextEditPluginNoState)
      .init_resource::<FeedbackFormState>()
      .init_resource::<FeedbackKeyCodeToggle>()
      .init_resource::<FeedbackPanelStyles>()
      .add_systems(
        Update,
        (
          spawn_feedback_ui.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          despawn_feedback_panel.run_if(resource_removed::<FeedbackPanelProps>),
          toggle_panel_visibility_with_key.run_if(resource_exists::<FeedbackKeyCodeToggle>),
          panel_visibility_sync.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          dropdown_visibility_sync.run_if(resource_exists_and_changed::<FeedbackFormState>),
          update_scroll_position,
          handle_hover_and_click_styles,
        )
          .run_if(resource_exists::<SessionApiKey>),
      );
  }
}
