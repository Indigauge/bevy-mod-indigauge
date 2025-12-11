use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
  event::{resources::BufferedEvents, systems::*},
  session::resources::SessionApiKey,
};

pub(crate) mod resources;
mod systems;
pub(crate) mod utils;

pub struct EventsPlugin {
  flush_interval: Duration,
}

impl EventsPlugin {
  pub fn new(flush_interval: Duration) -> Self {
    Self { flush_interval }
  }
}

impl Plugin for EventsPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        handle_queued_events,
        maybe_flush_events.run_if(resource_changed::<BufferedEvents>),
        flush_events.run_if(on_timer(self.flush_interval)),
      )
        .run_if(resource_exists::<SessionApiKey>),
    );
  }
}

pub mod macros {
  #[macro_export]
  macro_rules! enqueue_ig_event {
    ($level: ident, $etype:expr, $metadata:expr) => {
      const _VALID: &str = $crate::prelude::validate_event_type_compile_time($etype);
      let _ = $crate::prelude::enqueue(stringify!($level), $etype, $metadata, file!(), line!(), module_path!());
    };
  }

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
  ///   The value is compile-time validated by [`crate::utils::validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
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
  ///   The value is compile-time validated by [`crate::utils::validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
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
  ///   The value is compile-time validated by [`crate::utils::validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
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
  ///   The value is compile-time validated by [`crate::utils::validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
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
  ///   The value is compile-time validated by [`crate::utils::validate_event_type`] to ensure
  ///   it contains exactly one `.` and only letters on each side.
  /// * Optional metadata can be passed as a JSON-like key/value list.
  ///
  /// # Examples
  /// ```ignore
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
