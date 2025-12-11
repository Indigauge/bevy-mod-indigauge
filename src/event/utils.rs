use std::time::Instant;

use crate::{
  api_types::{EventPayload, EventPayloadCtx},
  event::resources::QueuedEvent,
  plugin::GLOBAL_TX,
  session::SESSION_START_INSTANT,
};

#[inline]
pub fn enqueue(
  level: &'static str,
  event_type: &str,
  metadata: Option<serde_json::Value>,
  file: &'static str,
  line: u32,
  module: &'static str,
) -> bool {
  let tx = match GLOBAL_TX.get() {
    Some(tx) => tx.clone(),
    None => return false,
  };

  if let Some(start_instant) = SESSION_START_INSTANT.get() {
    let elapsed_ms = Instant::now().duration_since(*start_instant).as_millis();
    let module = if module.is_empty() { None } else { Some(module) };

    let context = matches!(level, "warn" | "error").then(|| EventPayloadCtx {
      file: file.to_string(),
      line,
      module,
    });

    let payload = EventPayload {
      level,
      event_type: event_type.to_string(),
      elapsed_ms,
      metadata,
      idempotency_key: None,
      context,
    };

    tx.try_send(QueuedEvent::new(payload)).is_ok()
  } else {
    false
  }
}

pub const fn validate_event_type(s: &str) -> Result<(), &'static str> {
  let bytes = s.as_bytes();
  let len = bytes.len();

  if len < 3 {
    return Err("Invalid event type: too short (expected 'a.b')");
  }

  let mut dot_index: Option<usize> = None;
  let mut i = 0;

  while i < len {
    let b = bytes[i];
    match b {
      b'a'..=b'z' | b'A'..=b'Z' => {},
      b'.' => {
        if dot_index.is_some() {
          return Err("Invalid event type: multiple '.' found");
        }
        dot_index = Some(i);
      },
      _ => return Err("Invalid event type: only letters and a single '.' are allowed"),
    }
    i += 1;
  }

  let dot_pos = match dot_index {
    Some(p) => p,
    None => return Err("Invalid event type: must contain one '.'"),
  };

  if dot_pos == 0 || dot_pos == len - 1 {
    return Err("Invalid event type: '.' cannot be the first or last character");
  }

  Ok(())
}

/// Panics at compile time if the event type does not contain exactly one dot.
pub const fn validate_event_type_compile_time(s: &str) -> &str {
  if let Err(err) = validate_event_type(s) {
    panic!("{}", err);
  }
  s
}
