# Indigauge Game SDK (Bevy)

The **Indigauge Game SDK** is a lightweight Rust library for sending structured analytics and player feedback from your **Bevy** games to the [Indigauge](https://ingest.indigauge.com/v1/docs) API.

It’s designed to be easy to integrate and is powerful enough for production use in indie games.

---

## Features

- **Bevy 0.15 compatible** — easy drop-in plugin
- Lightweight event macros: `ig_info!`, `ig_warn!`, `ig_error!`, …
- Built-in **Feedback UI panel** for in-game bug reports & suggestions
- Works on both **native** and **WASM** builds

---

## Installation

```toml
[dependencies]
bevy = "0.15"
bevy-mod-indigauge = { version = "0.1.0" }
```

## Examples

- [`minimal`](examples/minimal.rs) - An example showing start session, sending info events and triggering feedback form.
- [`breakout`](examples/breakout.rs) – An example showing a more realistic setup with a real game and game states.

### Running Examples

```bash
cargo run --release --example minimal
```

```bash
INDIGAUGE_PUBLIC_KEY=YOUR_PUBLIC_KEY cargo run --release --example breakout
```

## Quick Start

* Setup game project [Indigauge](https://www.indigauge.com)
* Create a public key for the game.
* Add the plugin to your game.

```rust
use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_indigauge::{
  FeedbackCategory, FeedbackPanelProps, FeedbackPanelStyles, IndigaugeLogLevel, IndigaugeMode, IndigaugePlugin,
  StartSessionEvent, ig_info,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(
      IndigaugePlugin::new(
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
  commands.trigger(StartSessionEvent::new().with_locale("en-US").with_platform("steam"));
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
    // This is how you manually trigger the feedback panel
    commands.insert_resource(
      FeedbackPanelProps::with_question("What did you think about level 3?", FeedbackCategory::Gameplay)
        .allow_screenshot(false),
    );
  }
}

fn track_counter(mut counter: Local<u32>) {
  *counter += 1;
  ig_info!("counter.increase", { "value": *counter });
}
```

## Sending events

Send structured events with macros. The events will only be sent if a session was successfully started.

```rust
ig_info!("player.jump", { "height": 2.4 });
ig_error!("physics.failed", { "component": "rigid_body" });
```

## Bevy Compatibility

| bevy   | bevy-mod-indigauge |
| ------ | ---------------- |
| 0.15   | 0.1             |
