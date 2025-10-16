use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T, E = ErrorBody> {
  Ok(T),
  Err(E),
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
  pub code: String,
  pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponse {
  pub session_token: String,
}

impl StartSessionResponse {
  pub fn dev() -> Self {
    Self {
      session_token: Uuid::new_v4().to_string(),
    }
  }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionPayload<'a> {
  pub client_version: &'a str,
  pub player_id: Option<&'a String>,
  pub platform: Option<&'a String>,
  pub os: Option<&'a str>,
  pub cpu_family: Option<&'a String>,
  pub cores: Option<&'a str>,
  pub memory: Option<&'a str>,
  pub gpu: Option<&'a String>,
  pub locale: Option<&'a String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BatchEventPayload {
  pub events: Vec<EventPayload>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
  /// The type of the event. Event type must be in the format 'namespace.type'
  pub event_type: String,
  /// Metadata associated with the event.
  pub metadata: Option<serde_json::Value>,
  /// The level of the event.
  pub level: &'static str,
  /// Defaults to elapsed time since session start
  pub elapsed_ms: u128,
  /// Defaults to a random string
  pub idempotency_key: Option<String>,
  pub context: Option<EventPayloadCtx>,
}

#[derive(Serialize, Clone, Debug)]
pub struct EventPayloadCtx {
  pub file: String,
  pub line: u32,
  pub module: Option<&'static str>,
}
