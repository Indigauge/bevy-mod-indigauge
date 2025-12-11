# Indigauge Game SDK (Bevy)

The **Indigauge Game SDK** is a lightweight Rust library for sending structured analytics and player feedback from your **Bevy** games to the [Indigauge](https://ingest.indigauge.com/v1/docs) API.

It’s designed to be easy to integrate and is powerful enough for production use in indie games.

---

## Features

- **Bevy 0.15 compatible** — easy drop-in plugin
- Lightweight event macros: `ig_info!`, `ig_warn!`, `ig_error!`, …
- Built-in **Feedback UI panel** for in-game bug reports & suggestions
- Works on both **native** and **WASM** builds*
- **Tracing support** — log events to the Indigauge API through tracing

> [!WARNING]
> On wasm builds, the panic handler is disabled. No crash reports will be sent as events to the Indigauge API.

---

## Installation

```toml
[dependencies]
bevy = "0.15"
bevy-mod-indigauge = { version = "0.2" }
```

## Examples

- [`minimal`](examples/minimal.rs) - An example showing start session, sending info events and triggering feedback form.
- [`breakout`](examples/breakout.rs) – An example showing a more realistic setup with a real game and game states.

### Running Examples

```bash
cargo run --release --example minimal
```

```bash
cargo run --release --example breakout

# Or with tracing feature:
cargo run --release --example breakout --features tracing
```

## Quick Start

* Setup game project [Indigauge](https://www.indigauge.com)
* Create a public key for the game.
* Add the plugin to your game.

```rust
use std::time::Duration;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_mod_indigauge::prelude::{*, ig_info};

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
  commands.trigger(StartSessionEvent::new());
}

fn trigger_feedback_with_question(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
) {
  if keys.just_pressed(KeyCode::F) {
    // This is how you manually trigger the feedback panel
    commands.insert_resource(
      FeedbackPanelProps::with_question("What did you think about level 3?", FeedbackCategory::Gameplay),
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

## Tracing support

Send events to the Indigauge API through tracing. This is useful for debugging and monitoring your game.

### Enable the tracing feature

```toml
[dependencies]
bevy = { version = "0.15", features = ["bevy_mod_indigauge"] }
bevy-mod-indigauge = { version = "0.2", features = ["tracing"] }
```

```rust
use bevy::{log::{LogPlugin, BoxedLayer}, prelude::*};
use bevy_mod_indigauge::{prelude::*, tracing::IndigaugeLayer};

/// Default tracing layer, will send all events to the Indigauge API.
pub fn default_indigauge_layer(_app: &mut App) -> Option<BoxedLayer> {
  Some(Box::new(IndigaugeLayer::default()))
}

/// Custom tracing layer, will only send events that has an event_type, is either info, warn, or 
/// error and is not from the bevy_mod_othercrate module to the Indigauge API.
pub fn custom_indigauge_layer(_app: &mut App) -> Option<BoxedLayer> {
  Some(Box::new(
    IndigaugeLayer::default()
      .with_event_type_required(true) 
      .with_filters(vec!["bevy_mod_othercrate"])
      .with_levels(vec![
        IndigaugeLogLevel::Info,
        IndigaugeLogLevel::Warn,
        IndigaugeLogLevel::Error,
      ]),
  ))
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(LogPlugin {custom_layer: custom_indigauge_layer, ..default()}))
    .add_plugin(IndigaugePlugin::new("YOUR_PUBLIC_KEY", None, None))
    .add_systems(Startup, setup)
    .add_systems(Update, (track_counter.run_if(on_timer(Duration::from_secs(2)))))
    .run();
}

fn setup(mut commands: Commands) {
  commands.spawn((Camera2d, IsDefaultUiCamera));
  commands.trigger(StartSessionEvent::new());
}

fn track_counter(mut counter: Local<u32>) {
  *counter += 1;
  info!(ig = "counter.increase", value = *counter);
}
```

## Bevy Compatibility

| bevy   | bevy-mod-indigauge |
| ------ | ---------------- |
| 0.15   | 0.2             |
| 0.15   | 0.1             |
