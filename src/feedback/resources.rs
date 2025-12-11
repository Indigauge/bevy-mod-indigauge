use bevy::prelude::*;

use crate::feedback::types::{FeedbackCategory, FeedbackSpawnPosition};

#[derive(Resource, Debug)]
pub struct FeedbackKeyCodeToggle(pub KeyCode);

impl Default for FeedbackKeyCodeToggle {
  fn default() -> Self {
    Self(KeyCode::F2)
  }
}

#[derive(Resource, Debug)]
pub struct FeedbackPanelStyles {
  pub primary: Color,
  pub primary_hover: Color,
  pub secondary: Color,
  pub secondary_hover: Color,
  pub background: Color,
  pub surface: Color,
  pub border: Color,
  pub text_primary: Color,
  pub text_secondary: Color,
  pub success: Color,
  pub error: Color,
  pub warning: Color,
  pub accent: Color,
}

impl Default for FeedbackPanelStyles {
  fn default() -> Self {
    Self {
      primary: Color::srgb_u8(147, 164, 255),
      primary_hover: Color::srgb_u8(124, 140, 250),
      secondary: Color::srgb_u8(147, 164, 255),
      secondary_hover: Color::srgb_u8(124, 140, 250),
      background: Color::srgb_u8(15, 23, 42),
      surface: Color::srgb_u8(30, 41, 59),
      border: Color::srgb_u8(51, 65, 85),
      text_primary: Color::srgb_u8(248, 250, 252),
      text_secondary: Color::srgb_u8(203, 213, 225),
      success: Color::srgb_u8(34, 197, 94),
      error: Color::srgb_u8(248, 113, 113),
      warning: Color::srgb_u8(250, 204, 21),
      accent: Color::srgb_u8(168, 85, 247),
    }
  }
}

/// A [`Resource`] that controls the visibility and behavior of the in-game
/// feedback panel.
///
/// This resource is inserted into the [`World`](bevy::prelude::World) when you
/// want to show the feedback form manually. The [`IndigaugePlugin`](crate::prelude::IndigaugePlugin) listens
/// for it and renders a user interface that allows players to submit feedback
/// directly from within the game.
///
/// The feedback panel can optionally display a custom title and question, and
/// categorize the submitted feedback (for example, “Gameplay”, “UI”, or
/// “Performance”).
///
/// # Usage
///
/// You can insert this resource into the world at any time to display a feedback
/// form. For example:
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_mod_indigauge::prelude::*;
///
/// fn trigger_feedback_with_question(
///     mut commands: Commands,
///     keys: Res<ButtonInput<KeyCode>>,
/// ) {
///     if keys.just_pressed(KeyCode::Space) {
///         // Manually trigger the feedback panel with a predefined question.
///         commands.insert_resource(
///             FeedbackPanelProps::with_question(
///                 "What did you think about level 3?",
///                 FeedbackCategory::Gameplay,
///             ),
///         );
///     }
/// }
/// ```
///
/// # Fields
///
/// * `title` – Optional custom title shown at the top of the feedback form.
/// * `question` – Optional question to ask the player. Useful when you want
///   feedback about a specific moment or feature.
/// * `category` – Optional [`FeedbackCategory`] to tag the feedback (e.g.,
///   `Gameplay`, `UI`, `Bug`, etc.). If not provided, a category dropdown will be shown.
/// * `visible` – Whether the feedback panel is currently visible.
///   Managed automatically by the plugin.
/// * `allow_screenshot` – Whether the player should be allowed to attach
///   a screenshot with their feedback submission.
///
/// # See Also
///
/// * [`FeedbackCategory`] – Defines available categories for feedback.
///
#[derive(Resource)]
pub struct FeedbackPanelProps {
  /// Optional custom title shown at the top of the feedback form.
  pub(crate) title: Option<String>,

  /// Optional question to ask the player.
  pub(crate) question: Option<String>,

  /// The category to tag the feedback with (e.g., Gameplay, UI, Bug).
  pub(crate) category: Option<FeedbackCategory>,

  /// Whether the feedback panel is currently visible.
  pub visible: bool,

  /// Whether screenshots are allowed when submitting feedback.
  pub(crate) allow_screenshot: bool,

  pub(crate) spawn_position: FeedbackSpawnPosition,

  pub(crate) position_margin: UiRect,
}

impl Default for FeedbackPanelProps {
  /// Creates a new feedback panel with "Send feedback" as title, no question and no category.
  /// Screenshots are allowed by default.
  fn default() -> Self {
    Self {
      title: Some("Send feedback".to_string()),
      question: None,
      category: None,
      visible: true,
      allow_screenshot: true,
      spawn_position: FeedbackSpawnPosition::default(),
      position_margin: UiRect::all(Val::Px(16.0)),
    }
  }
}

impl FeedbackPanelProps {
  /// Creates a new feedback panel with a question and a fixed category.
  /// Screenshots are not allowed by default.
  pub fn with_question(question: impl Into<String>, category: FeedbackCategory) -> Self {
    Self {
      title: None,
      question: Some(question.into()),
      category: Some(category),
      visible: true,
      allow_screenshot: false,
      ..Default::default()
    }
  }

  /// Set a custom title for the feedback panel
  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
    self
  }

  pub fn allow_screenshot(mut self, allow_screenshot: bool) -> Self {
    self.allow_screenshot = allow_screenshot;
    self
  }

  pub fn spawn_position(mut self, spawn_position: FeedbackSpawnPosition) -> Self {
    self.spawn_position = spawn_position;
    self
  }

  pub fn margin(mut self, margin: UiRect) -> Self {
    self.position_margin = margin;
    self
  }
}

#[derive(Resource, Default)]
pub struct FeedbackFormState {
  // pub rating: u8,                 // 1..=5
  pub category: FeedbackCategory, // dropdown-valg
  pub include_screenshot: bool,
  pub dropdown_open: bool,
  pub question: Option<String>,
  pub error: Option<String>,
}

#[derive(Resource)]
pub struct TakeScreenshot;
