use crossbeam_channel::Sender;
use once_cell::sync::OnceCell;
use std::time::Instant;

use crate::resources::events::QueuedEvent;

mod api_types;
mod observers;
pub(crate) mod plugins;
pub(crate) mod primitives;
pub(crate) mod resources;
pub(crate) mod sysparam;
pub(crate) mod systems;
pub mod utils;

#[cfg(feature = "tracing")]
pub mod tracing;

pub(crate) static GLOBAL_TX: OnceCell<Sender<QueuedEvent>> = OnceCell::new();
pub(crate) static SESSION_START_INSTANT: OnceCell<Instant> = OnceCell::new();

pub mod prelude {
  

  pub use crate::observers::feedback::{switch_state_on_feedback_despawn, switch_state_on_feedback_spawn};
  pub use crate::observers::session::switch_state_after_session_init;
  pub use crate::plugins::IndigaugePlugin;
  pub use crate::primitives::feedback::FeedbackCategory;
  pub use crate::primitives::{IndigaugeInitDoneEvent, session::StartSessionEvent};
  pub use crate::resources::feedback::{
    FeedbackKeyCodeToggle, FeedbackPanelProps, FeedbackPanelStyles, FeedbackSpawnPosition,
  };
  pub use crate::resources::{IndigaugeLogLevel, IndigaugeMode, session::EmptySessionMeta};
  pub use crate::systems::session::{end_session, start_default_session};
}
pub mod macros {
  #[macro_export]
  macro_rules! enqueue_ig_event {
    ($level: ident, $etype:expr, $metadata:expr) => {
      const _VALID: &str = $crate::utils::validate_event_type_compile_time($etype);
      let _ = $crate::utils::enqueue(stringify!($level), $etype, $metadata, file!(), line!(), module_path!());
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
