use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum FeedbackCategory {
  #[default]
  General,
  Ui,
  Gameplay,
  Performance,
  Bugs,
  Controls,
  Audio,
  Balance,
  Graphics,
  Visual,
  Art,
  Other,
}

impl FeedbackCategory {
  pub const ALL: &'static [FeedbackCategory] = &[
    FeedbackCategory::General,
    FeedbackCategory::Ui,
    FeedbackCategory::Gameplay,
    FeedbackCategory::Performance,
    FeedbackCategory::Bugs,
    FeedbackCategory::Controls,
    FeedbackCategory::Audio,
    FeedbackCategory::Balance,
    FeedbackCategory::Graphics,
    FeedbackCategory::Visual,
    FeedbackCategory::Art,
    FeedbackCategory::Other,
  ];

  pub fn label(&self) -> &'static str {
    match self {
      FeedbackCategory::General => "General",
      FeedbackCategory::Ui => "UI",
      FeedbackCategory::Gameplay => "Gameplay",
      FeedbackCategory::Performance => "Performance",
      FeedbackCategory::Bugs => "Bugs",
      FeedbackCategory::Other => "Other",
      FeedbackCategory::Controls => "Controls",
      FeedbackCategory::Audio => "Audio",
      FeedbackCategory::Balance => "Balance",
      FeedbackCategory::Graphics => "Graphics",
      FeedbackCategory::Visual => "Visual",
      FeedbackCategory::Art => "Art",
    }
  }
}

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
