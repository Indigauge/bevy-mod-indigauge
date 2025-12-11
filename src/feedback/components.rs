use bevy::prelude::*;

use crate::feedback::types::FeedbackCategory;

#[derive(Component)]
pub struct OriginalButtonStyles {
  pub background: Color,
  pub border: Color,
}

#[derive(Component)]
pub struct ButtonHoverStyle {
  pub background: Color,
  pub border: Color,
}

#[derive(Component)]
pub struct ButtonPressedStyle {
  pub background: Color,
  pub border: Color,
}

#[derive(Component)]
pub struct FeedbackPanel;

#[derive(Component)]
pub struct MessageInput;

#[derive(Component)]
pub struct QuestionTextRoot;

// #[derive(Component)]
// pub struct RatingStar(pub u8);

#[derive(Component)]
pub struct CategoryButton;

#[derive(Component)]
pub struct CategoryList; // container som toggles

#[derive(Component)]
pub struct CategoryItem(pub FeedbackCategory);

#[derive(Component)]
pub struct SubmitButton;

#[derive(Component)]
pub struct CancelButton;

#[derive(Component)]
#[require(HoldPressed)]
pub struct ScreenshotToggle;

#[derive(Component)]
pub struct ScreenshotToggleText;

#[derive(Component)]
pub struct CategoryButtonText;

#[derive(Component, Default)]
pub struct HoldPressed;

#[derive(Component)]
pub struct Active;
