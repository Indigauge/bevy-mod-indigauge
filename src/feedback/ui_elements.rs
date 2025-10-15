use bevy::prelude::*;

use crate::feedback::FeedbackCategory;

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

pub struct FeedbackPanelCard;

#[derive(Component)]
pub struct FeedbackPanel;

#[derive(Component)]
#[require(InputState, HoldPressed)]
pub struct MessageInput;

#[derive(Component)]
pub struct MessageTextRoot;

#[derive(Component)]
pub struct PlaceholderTextRoot;

#[derive(Component)]
pub struct RatingStar(pub u8);

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

#[derive(Component, Default)]
pub struct InputState {
  pub focused: bool,
  pub content: String,
}

pub fn panel(background_color: Color, border_color: Color) -> impl Bundle {
  (BorderRadius::all(Val::Px(8.0)), BackgroundColor(background_color), BorderColor(border_color))
}

pub fn button(background_color: Color, border_color: Color) -> impl Bundle {
  (
    Button,
    BorderRadius::all(Val::Px(8.0)),
    BackgroundColor(background_color),
    BorderColor(border_color),
    ButtonHoverStyle {
      background: background_color.with_alpha(0.5),
      border: border_color.with_alpha(0.5),
    },
    ButtonPressedStyle {
      background: background_color.with_alpha(0.2),
      border: border_color.with_alpha(0.2),
    },
  )
}
