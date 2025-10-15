// feedback_ui.rs
use bevy::prelude::*;
use bevy_text_edit::TextEditPluginNoState;

use crate::feedback::{
  resources::{FeedbackFormState, FeedbackKeyCodeToggle, FeedbackPanelStyles},
  systems::{
    category_pick_system, category_toggle_system, dropdown_visibility_sync, handle_hover_and_click_styles,
    panel_visibility_sync, screenshot_toggle_click_system, spawn_feedback_ui, submit_click_system,
    toggle_panel_visibility_with_key, update_scroll_position,
  },
  ui_elements::*,
};

pub mod resources;
mod systems;
pub mod ui_elements;

pub use resources::FeedbackPanelProps;

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
          toggle_panel_visibility_with_key.run_if(resource_exists::<FeedbackKeyCodeToggle>),
          panel_visibility_sync.run_if(resource_exists_and_changed::<FeedbackPanelProps>),
          category_toggle_system,
          category_pick_system,
          dropdown_visibility_sync.run_if(resource_exists_and_changed::<FeedbackFormState>),
          screenshot_toggle_click_system,
          submit_click_system,
          update_scroll_position,
          handle_hover_and_click_styles,
        ),
      );
  }
}
