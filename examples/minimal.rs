use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_indigauge::{
  ig_info,
  prelude::{
    EmptySessionMeta, FeedbackCategory, FeedbackPanelProps, FeedbackPanelStyles, IndigaugeLogLevel, IndigaugeMode,
    IndigaugePlugin, StartSessionEvent,
  },
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(
      IndigaugePlugin::<EmptySessionMeta>::new(
        "YOUR_PUBLIC_KEY",
        Some("Your game name (defaults to `CARGO_PKG_NAME` if not provided)".to_string()),
        Some("Your game version (defaults to `CARGO_PKG_VERSION` if not provided)".to_string())
      )
      // Optional: Set mode (Defaults to live). Dev mode is useful for testing and debugging and does not send events to the server.
      .mode(IndigaugeMode::Dev)
      // Optional: Set preferred log-level (Defaults to Info)
      .log_level(IndigaugeLogLevel::Info)
    )
    // Optional: Customize the feedback panel styles
    .insert_resource(FeedbackPanelStyles {
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
    })
    .add_systems(Startup, setup)
    .add_systems(Update, (trigger_feedback_with_question, track_counter.run_if(on_timer(Duration::from_secs(2)))))
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));
  commands.trigger(StartSessionEvent::new().with_platform("steam"));

  const HELP_TEXT_DEFAULT: &str = "Press 'F2' to display the default feedback panel!\n";
  const HELP_TEXT_WITH_QUESTION: &str = "Press 'SPACE' to display the feedback panel with a question!\n";

  commands
    .spawn(Node {
      flex_direction: FlexDirection::Column,
      row_gap: Val::Px(8.0),
      ..default()
    })
    .with_children(|builder| {
      builder.spawn(Text::new(HELP_TEXT_DEFAULT));
      builder.spawn(Text::new(HELP_TEXT_WITH_QUESTION));
    });
}

fn trigger_feedback_with_question(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  existing: Option<Res<FeedbackPanelProps>>,
) {
  if existing.is_some() {
    return;
  }

  if keys.just_pressed(KeyCode::Space) {
    commands.insert_resource(FeedbackPanelProps::with_question(
      "What did you think about level 3?",
      FeedbackCategory::Gameplay,
    ));
  }
}

fn track_counter(mut counter: Local<u32>) {
  *counter += 1;
  ig_info!("counter.increase", { "value": *counter });
}
