use std::{
  env::consts::OS,
  ops::Deref,
  time::{Duration, Instant},
};

use bevy::{
  ecs::system::{IntoObserverSystem, ObserverSystem},
  prelude::*,
  time::common_conditions::on_timer,
  window::WindowCloseRequested,
};
use bevy_mod_reqwest::{BevyReqwest, JsonResponse, ReqwestErrorEvent, ReqwestPlugin, ReqwestResponseEvent};
use crossbeam_channel::{Receiver, Sender, bounded};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
  api_types::{ApiResponse, EventPayload, EventPayloadCtx, StartSessionPayload, StartSessionResponse},
  feedback::{FeedbackPanelVisible, FeedbackUiPlugin},
  observers::{observe_start_session_event, on_start_session_error, on_start_session_response},
  resources::{
    IndigaugeConfig, IndigaugeLogLevel, LastSentRequestInstant, SessionApiKey,
    events::{BufferedEvents, EventQueueReceiver, QueuedEvent},
  },
  sysparam::BevyIndigauge,
};

mod api_types;
pub mod feedback;
mod observers;
pub mod resources;
pub mod sysparam;
pub mod utils;

pub(crate) static GLOBAL_TX: OnceCell<Sender<QueuedEvent>> = OnceCell::new();
pub(crate) static SESSION_START_INSTANT: OnceCell<Instant> = OnceCell::new();
const SESSION_KEY_EXPIRY_MS: u128 = 6 * 60 * 60 * 1000;

#[derive(Event, Default)]
pub struct StartSessionEvent {
  pub locale: Option<String>,
  pub platform: Option<String>,
}

#[derive(Event, Debug)]
pub enum IndigaugeInitDoneEvent {
  Success,
  Skipped(String),
  Failure(String),
  UnexpectedFailure(String),
}

pub struct IndigaugePlugin {
  public_key: String,
  /// Defaults to cargo package name
  game_name: String,
  game_version: String,
  enabled: bool,
  log_level: IndigaugeLogLevel,
}

impl IndigaugePlugin {
  pub fn new(public_key: String, game_name: Option<String>, game_version: String) -> Self {
    Self {
      public_key,
      game_name: game_name.unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string()),
      game_version,
      ..Default::default()
    }
  }

  pub fn log_level(mut self, log_level: IndigaugeLogLevel) -> Self {
    self.log_level = log_level;
    self
  }

  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }
}

impl Default for IndigaugePlugin {
  fn default() -> Self {
    Self {
      game_name: env!("CARGO_PKG_NAME").to_string(),
      public_key: std::env::var("INDIGAUGE_PUBLIC_KEY").unwrap_or_else(|_| {
        warn!("INDIGAUGE_PUBLIC_KEY environment variable not set");
        "".to_string()
      }),
      game_version: env!("CARGO_PKG_VERSION").to_string(),
      enabled: true,
      log_level: IndigaugeLogLevel::Info,
    }
  }
}

impl Plugin for IndigaugePlugin {
  fn build(&self, app: &mut App) {
    let config = IndigaugeConfig::new(&self.game_name, &self.public_key, &self.game_version);

    if self.enabled {
      if config.public_key.is_empty() {
        if self.log_level <= IndigaugeLogLevel::Warn {
          warn!("Indigauge public key is not set");
        }
      } else {
        if GLOBAL_TX.get().is_none() {
          let (tx, rx) = bounded::<QueuedEvent>(config.max_queue);
          GLOBAL_TX.set(tx).ok();

          app.insert_resource(EventQueueReceiver::new(rx));
        }
      }
    }

    app
      .add_plugins(ReqwestPlugin::default())
      .add_plugins(FeedbackUiPlugin)
      .insert_resource(FeedbackPanelVisible(true))
      .add_event::<StartSessionEvent>()
      .insert_resource(self.log_level.clone())
      .insert_resource(BufferedEvents::default())
      .insert_resource(LastSentRequestInstant::new())
      .add_observer(observe_start_session_event)
      .add_systems(
        Update,
        (
          handle_queued_events,
          maybe_flush_events.run_if(resource_changed::<BufferedEvents>),
          flush_events.run_if(on_timer(config.flush_interval.clone())),
        )
          .run_if(resource_exists::<SessionApiKey>),
      )
      .insert_resource(config)
      .add_systems(
        PostUpdate,
        (handle_exit_event::<AppExit>, handle_exit_event::<WindowCloseRequested>)
          .run_if(resource_exists::<SessionApiKey>),
      );
  }
}

fn maybe_flush_events(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  if ig.buffered_events.events.len() >= ig.config.batch_size {
    ig.flush_events(&*session_key);
  }
}

fn flush_events(mut ig: BevyIndigauge, session_key: Res<SessionApiKey>) {
  if ig.flush_events(&*session_key) == 0 {
    ig.send_heartbeat(&*session_key);
  }
}

fn handle_exit_event<E>(mut exit_events: EventReader<E>, mut ig: BevyIndigauge, session_key: Res<SessionApiKey>)
where
  E: Event + std::fmt::Debug,
{
  exit_events.read().for_each(|_event| {
    let reqwest_client = ig.build_request("sessions/end", &session_key, &json!({"reason": "ended"}));

    if let Ok(reqwest_client) = reqwest_client {
      ig.reqwest_client.send(reqwest_client);
    }

    ig.flush_events(&*session_key);
  });
}

fn handle_queued_events(
  receiver: Res<EventQueueReceiver>,
  mut buffered_events: ResMut<BufferedEvents>,
  log_level: Res<IndigaugeLogLevel>,
) {
  for event in receiver.try_iter() {
    match event.validate() {
      Ok(_) => {
        buffered_events.events.push(event);
      },
      Err(err) => {
        if *log_level <= IndigaugeLogLevel::Error {
          error!("Invalid event: {}", err);
        }
      },
    }
  }
}

/* ===========================
Makroer – tracing-lignende
=========================== */

mod internal_macros {
  #[macro_export]
  macro_rules! enqueue_ig_event {
    ($level: ident, $etype:expr, $metadata:expr) => {
      const _VALID: &str = $crate::utils::validate_event_type($etype);
      let _ = $crate::utils::enqueue(stringify!($level), $etype, $metadata, file!(), line!(), module_path!());
    };
  }
}

pub mod macros {
  /// Usage example: ig_event!(info, "ui.click", { "button": btn_id, "x": x, "y": y });
  #[macro_export]
  macro_rules! ig_event {
    ($level:ident, $etype:expr $(,)?) => {{
      $crate::enqueue_ig_event!($level, $etype, None);
    }};
    ($level:ident, $etype:expr $(, { $($key:tt : $value:expr),* $(,)? })? ) => {{
      let meta = serde_json::json!({ $($($key : $value),*)? });
      $crate::enqueue_ig_event!($level, $etype, Some(meta));
    }};
  }

  /// Logs or enqueues a **trace-level** event to Indigauge.
  ///
  /// # Format
  /// ```ignore
  /// ig_trace!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as `"namespace.event"`,
  ///   e.g. `"ui.click"`, `"gameplay.start"`.
  ///   The value is compile-time validated by [`validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```rust
  /// ig_trace!("ui.hover");
  /// ig_trace!("ui.hover", { "x": 128, "y": 256 });
  /// ```
  #[macro_export]
  macro_rules! ig_trace {
      ($($tt:tt)*) => { $crate::ig_event!(trace, $($tt)*); }
  }

  /// Logs or enqueues a **debug-level** event to Indigauge.
  ///
  /// # Format
  /// ```ignore
  /// ig_debug!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as `"namespace.event"`,
  ///   e.g. `"ui.click"`, `"gameplay.start"`.
  ///   The value is compile-time validated by [`validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```rust
  /// ig_debug!("system.update_start");
  /// ig_debug!("pathfinding.step", { "node": node_id, "distance": dist });
  /// ```
  #[macro_export]
  macro_rules! ig_debug {
      ($($tt:tt)*) => { $crate::ig_event!(debug, $($tt)*); }
  }

  /// Logs or enqueues an **info-level** event to Indigauge.
  ///
  /// Used for general application telemetry that represents normal operation.
  ///
  /// # Format
  /// ```ignore
  /// ig_info!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as `"namespace.event"`,
  ///   e.g. `"ui.click"`, `"gameplay.start"`.
  ///   The value is compile-time validated by [`validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```rust
  /// ig_info!("gameplay.start");
  /// ig_info!("gameplay.start", { "session": session_id });
  /// ig_info!("ui.click", { "button": "play" });
  /// ```
  #[macro_export]
  macro_rules! ig_info {
      ($($tt:tt)*) => { $crate::ig_event!(info, $($tt)*); }
  }

  /// Logs or enqueues a **warning-level** event to Indigauge.
  ///
  /// Used for non-fatal issues that should be visible in dashboards or analytics.
  ///
  /// # Format
  /// ```ignore
  /// ig_warn!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as `"namespace.event"`,
  ///   e.g. `"ui.click"`, `"gameplay.start"`.
  ///   The value is compile-time validated by [`validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```rust
  /// ig_warn!("network.latency", { "ping_ms": latency });
  /// ig_warn!("save.failed", { "reason": "disk_full" });
  /// ```
  #[macro_export]
  macro_rules! ig_warn {
      ($($tt:tt)*) => { $crate::ig_event!(warn, $($tt)*); }
  }

  /// Logs or enqueues an **error-level** event to Indigauge.
  ///
  /// Used to capture errors, failures, or critical analytics signals.
  ///
  /// # Format
  /// ```ignore
  /// ig_error!(<event_type> [, { <metadata_key>: <value>, ... }]);
  /// ```
  ///
  /// * `<event_type>` — must be a string literal formatted as `"namespace.event"`,
  ///   e.g. `"ui.click"`, `"gameplay.start"`.
  ///   The value is compile-time validated by [`validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```rust
  /// ig_error!("game.crash", { "reason": error_message });
  /// ig_error!("network.disconnect");
  /// ```
  ///
  /// The metadata is optional, but recommended for debugging or later filtering.
  #[macro_export]
  macro_rules! ig_error {
      ($($tt:tt)*) => { $crate::ig_event!(error, $($tt)*); }
  }
}
