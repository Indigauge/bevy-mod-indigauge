pub(crate) fn bucket_cores(n: u32) -> &'static str {
  match n {
    0..=2 => "1-2",
    3..=4 => "3-4",
    5..=8 => "6-8",
    _ => ">8",
  }
}

pub(crate) fn bucket_ram_gb(gb: u32) -> &'static str {
  match gb {
    0..=4 => "<=4",
    5..=8 => "6-8",
    9..=16 => "12-16",
    _ => ">16",
  }
}

/// Coarsens a CPU model string (e.g. from Bevy’s `SystemInfo`) into a privacy-safe, general category.
/// Intended for analytics and diagnostics, *not* fingerprinting.
///
/// Examples:
/// - "11th Gen Intel(R) Core(TM) i7-11850H @ 2.50GHz" → "Intel i7 11th Gen"
/// - "AMD Ryzen 7 5800X3D 8-Core Processor"           → "AMD Ryzen 7 5000 Series"
/// - "Apple M2 Pro"                                   → "Apple M2"
/// - "Intel(R) Xeon(R) CPU E5-2678 v3"                → "Intel Xeon"
pub(crate) fn coarsen_cpu_name(cpu_raw: &str) -> Option<String> {
  let name = cpu_raw.to_ascii_lowercase();

  // Apple Silicon
  if name.contains("apple m1") {
    return Some("Apple M1".into());
  } else if name.contains("apple m2") {
    return Some("Apple M2".into());
  } else if name.contains("apple m3") {
    return Some("Apple M3".into());
  }

  // Intel families
  if name.contains("intel") {
    if name.contains("celeron") {
      return Some("Intel Celeron".into());
    }
    if name.contains("pentium") {
      return Some("Intel Pentium".into());
    }
    if name.contains("xeon") {
      return Some("Intel Xeon".into());
    }
    if name.contains("atom") {
      return Some("Intel Atom".into());
    }
    if name.contains("core") {
      if name.contains("i3") {
        return Some(if let Some(generation) = extract_generation(&name) {
          format!("Intel i3 {}th Gen", generation)
        } else {
          "Intel i3".into()
        });
      } else if name.contains("i5") {
        return Some(if let Some(generation) = extract_generation(&name) {
          format!("Intel i5 {}th Gen", generation)
        } else {
          "Intel i5".into()
        });
      } else if name.contains("i7") {
        return Some(if let Some(generation) = extract_generation(&name) {
          format!("Intel i7 {}th Gen", generation)
        } else {
          "Intel i7".into()
        });
      } else if name.contains("i9") {
        return Some(if let Some(generation) = extract_generation(&name) {
          format!("Intel i9 {}th Gen", generation)
        } else {
          "Intel i9".into()
        });
      } else {
        return Some("Intel Core (Other)".into());
      }
    }
    // fallback for Intel
    return Some("Intel (Other)".into());
  }

  // AMD families
  if name.contains("amd") {
    if name.contains("ryzen") {
      if name.contains("threadripper") {
        return Some("AMD Ryzen Threadripper".into());
      }
      // detect generation (e.g., 7000, 5000, 3000)
      if let Some(generation) = extract_ryzen_gen(&name) {
        return Some(format!("AMD Ryzen {} Series", generation));
      }
      return Some("AMD Ryzen".into());
    }
    if name.contains("epyc") {
      return Some("AMD EPYC".into());
    }
    if name.contains("athlon") {
      return Some("AMD Athlon".into());
    }
    // fallback for AMD
    return Some("AMD (Other)".into());
  }

  // ARM (non-Apple)
  if name.contains("arm") || name.contains("cortex") {
    return Some("ARM (Generic)".into());
  }

  None
}

/// Extracts Intel generation number from a lowercase CPU string like "11th gen intel core i7-11800h".
fn extract_generation(name: &str) -> Option<u8> {
  // "11th gen" → 11
  if let Some(pos) = name.find("th gen") {
    // find preceding number
    let before = &name[..pos];
    let digits: String = before.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
      return digits.chars().rev().collect::<String>().parse().ok();
    }
  }
  // fallback: try parse from model number (e.g. "i7-11800h" → 11)
  if let Some(pos) = name.find("i7-")
    && name.len() > pos + 3
    && let Some(d) = name[pos + 3..].chars().next()
  {
    return d.to_digit(10).map(|v| v as u8);
  }
  None
}

/// Extracts Ryzen generation (3/5/7/9/5000 etc.)
fn extract_ryzen_gen(name: &str) -> Option<String> {
  // Match e.g. "ryzen 7 5800x" → "5000"
  let parts: Vec<&str> = name.split_whitespace().collect();
  for (i, part) in parts.iter().enumerate() {
    if *part == "ryzen" && i + 2 < parts.len() {
      let next = parts[i + 2];
      // First digit of model number gives series (5 → 5000)
      if let Some(first_digit) = next.chars().next()
        && first_digit.is_ascii_digit()
      {
        return Some(format!("{}000", first_digit));
      }
    }
  }
  None
}

#[cfg(all(feature = "panic_handler", not(target_family = "wasm")))]
pub fn panic_handler(
  host_origin: String,
  session_api_key: String,
) -> impl Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static {
  use crate::{
    api_types::{EventPayload, EventPayloadCtx},
    session::SESSION_START_INSTANT,
  };
  use serde_json::json;
  use std::time::Instant;

  move |info| {
    if let Some(start_instant) = SESSION_START_INSTANT.get() {
      use crate::api_types::StartSessionResponse;
      if session_api_key == StartSessionResponse::dev().session_token {
        return;
      }

      let elapsed_ms = Instant::now().duration_since(*start_instant).as_millis();

      let metadata = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| json!({"message": s.to_string()}));

      let context = info.location().map(|loc| EventPayloadCtx {
        file: loc.file().to_string(),
        line: loc.line(),
        module: None,
      });
      let payload = EventPayload {
        level: "fatal",
        event_type: "game.crash".to_string(),
        elapsed_ms,
        metadata,
        idempotency_key: None,
        context,
      };

      let single_event_endpoint = format!("{}/v1/events", host_origin);
      let client = reqwest::blocking::Client::new();
      let _ = client
        .post(&single_event_endpoint)
        .header("X-Indigauge-Key", &session_api_key)
        .json(&payload)
        .send();

      let end_session_endpoint = format!("{}/v1/sessions/end", host_origin);
      let _ = client
        .post(&end_session_endpoint)
        .header("X-Indigauge-Key", &session_api_key)
        .json(&json!({"reason": "crashed"}))
        .send();
    }
  }
}
