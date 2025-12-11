use once_cell::sync::OnceCell;
use std::time::Instant;

mod api_types;
pub mod utils;

pub(crate) mod config;
pub(crate) mod event;
pub(crate) mod feedback;
pub mod plugin;
pub(crate) mod session;

#[cfg(feature = "tracing")]
pub mod tracing;

pub(crate) static SESSION_START_INSTANT: OnceCell<Instant> = OnceCell::new();

pub mod prelude {
  pub use crate::config::{IndigaugeLogLevel, IndigaugeMode};
  pub use crate::event::utils::{enqueue, validate_event_type_compile_time};
  pub use crate::feedback::observers::{switch_state_on_feedback_despawn, switch_state_on_feedback_spawn};
  pub use crate::feedback::{
    resources::{FeedbackKeyCodeToggle, FeedbackPanelProps, FeedbackPanelStyles},
    types::{FeedbackCategory, FeedbackSpawnPosition},
  };
  pub use crate::plugin::IndigaugePlugin;
  pub use crate::session::observers::switch_state_after_session_init;
  pub use crate::session::systems::{end_session, start_default_session};
  pub use crate::session::{
    events::{IndigaugeInitDoneEvent, StartSessionEvent},
    resources::EmptySessionMeta,
  };
}
