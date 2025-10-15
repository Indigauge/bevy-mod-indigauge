use bevy::prelude::*;

use crate::feedback::FeedbackCategory;

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

#[derive(Resource, Default)]
pub struct FeedbackPanelProps {
  pub(crate) question: Option<String>,
  pub(crate) category: Option<FeedbackCategory>,
  pub(crate) visible: bool,
  pub(crate) allow_screenshot: bool,
}

impl FeedbackPanelProps {
  pub fn with_question(question: impl Into<String>, category: FeedbackCategory) -> Self {
    Self {
      question: Some(question.into()),
      category: Some(category),
      visible: true,
      allow_screenshot: true,
    }
  }

  pub fn visible() -> Self {
    Self {
      question: None,
      category: None,
      visible: true,
      allow_screenshot: true,
    }
  }

  pub fn allow_screenshot(mut self, allow_screenshot: bool) -> Self {
    self.allow_screenshot = allow_screenshot;
    self
  }
}

#[derive(Resource, Default)]
pub struct FeedbackFormState {
  pub rating: u8,                 // 1..=5
  pub category: FeedbackCategory, // dropdown-valg
  pub include_screenshot: bool,
  pub dropdown_open: bool,
  pub question: Option<String>,
}
