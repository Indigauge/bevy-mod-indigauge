use crate::event::utils::{enqueue, validate_event_type};
use crate::prelude::IndigaugeLogLevel;
use bevy::utils::tracing::{Event, Subscriber, field::Field};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing_subscriber::{Layer, field::Visit, layer::Context, registry::LookupSpan};

const EVENT_TYPE_FIELDS: &[&str] = &["ig", "event_type"];

trait IndigaugeSink: Send + Sync + 'static {
  fn log(
    &self,
    level: &'static str,
    event_type: &str,
    metadata: Option<serde_json::Value>,
    file: &'static str,
    line: u32,
    module: &'static str,
  );
}

struct EnqueueSink;

#[cfg(feature = "tracing")]
impl IndigaugeSink for EnqueueSink {
  fn log(
    &self,
    level: &'static str,
    event_type: &str,
    metadata: Option<serde_json::Value>,
    file: &'static str,
    line: u32,
    module: &'static str,
  ) {
    enqueue(level, event_type, metadata, file, line, module);
  }
}

#[derive(Default, Debug)]
struct FieldVisitor {
  event_type: Option<String>,
  fields: BTreeMap<String, Value>,
}

impl Visit for FieldVisitor {
  fn record_str(&mut self, field: &Field, value: &str) {
    let name = field.name();

    if self.event_type.is_none() && EVENT_TYPE_FIELDS.contains(&name) {
      if validate_event_type(value).is_ok() {
        self.event_type = Some(value.to_string());
      }
    } else {
      self.fields.insert(name.to_string(), Value::String(value.to_string()));
    }
  }

  fn record_bool(&mut self, field: &Field, value: bool) {
    let name = field.name();
    self.fields.insert(name.to_string(), Value::Bool(value));
  }

  fn record_u64(&mut self, field: &Field, value: u64) {
    let name = field.name();
    self.fields.insert(name.to_string(), json!(value));
  }

  fn record_i64(&mut self, field: &Field, value: i64) {
    let name = field.name();
    self.fields.insert(name.to_string(), json!(value));
  }

  fn record_f64(&mut self, field: &Field, value: f64) {
    let name = field.name();
    self.fields.insert(name.to_string(), json!(value));
  }

  fn record_u128(&mut self, field: &Field, value: u128) {
    let name = field.name();
    self.fields.insert(name.to_string(), json!(value));
  }

  fn record_i128(&mut self, field: &Field, value: i128) {
    let name = field.name();
    self.fields.insert(name.to_string(), json!(value));
  }

  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    let name = field.name();
    let s = format!("{value:?}");

    self.fields.insert(name.to_string(), Value::String(s));
  }
}

pub struct IndigaugeLayer {
  filters: Vec<String>,
  levels: Vec<IndigaugeLogLevel>,
  event_type_required: bool,
  sink: Arc<dyn IndigaugeSink>,
}

impl Default for IndigaugeLayer {
  fn default() -> Self {
    Self {
      filters: vec!["bevy_mod_indigauge".to_string()],
      levels: vec![
        IndigaugeLogLevel::Info,
        IndigaugeLogLevel::Warn,
        IndigaugeLogLevel::Error,
      ],
      event_type_required: false,
      sink: Arc::new(EnqueueSink),
    }
  }
}

impl IndigaugeLayer {
  /// Set event type required for the layer.
  ///
  /// If true this will filter out events without the required event type, so that
  /// events without an `ig` or `event_type` field will be filtered out. This allows
  /// for more precise filtering of events and can be used to filter out events that
  /// are not relevant to log to indigauge.
  ///
  /// Default is false. All events will be logged, but events without an `ig` or `event_type` field will
  /// get event type `tracing.<Level>`. Ie: `tracing.info`, `tracing.warn`, `tracing.error`.
  ///
  /// # Example
  /// ```
  /// use bevy_mod_indigauge::{tracing::IndigaugeLayer, prelude::IndigaugeLogLevel};
  ///
  /// let layer = IndigaugeLayer::default().with_event_type_required(true);
  /// ```
  pub fn with_event_type_required(mut self, event_type_required: bool) -> Self {
    self.event_type_required = event_type_required;
    self
  }

  /// Set filters for the layer. This will filter out events with modules not in the provided list.
  ///
  /// The filter is a string that matches the module path of the event.
  /// Obs: The filter uses `starts_with` to match the module path.
  ///
  /// # Example
  /// ```
  /// use bevy_mod_indigauge::tracing::IndigaugeLayer;
  ///
  /// let layer = IndigaugeLayer::default().with_filters(vec!["bevy_mod_indigauge"]);
  /// ```
  pub fn with_filters<T>(mut self, filters: Vec<T>) -> Self
  where
    T: Into<String>,
  {
    filters.into_iter().for_each(|filter| self.filters.push(filter.into()));
    self
  }

  /// Set levels for the layer. This will filter out events with levels not in the provided list.
  ///
  /// # Example
  /// ```
  /// use bevy_mod_indigauge::{tracing::IndigaugeLayer, prelude::IndigaugeLogLevel};
  ///
  /// let layer = IndigaugeLayer::default().with_levels(vec![IndigaugeLogLevel::Error, IndigaugeLogLevel::Warn]);
  /// ```
  pub fn with_levels(mut self, levels: Vec<IndigaugeLogLevel>) -> Self {
    self.levels = levels;
    self
  }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for IndigaugeLayer {
  fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
    let metadata = event.metadata();
    let module = metadata.module_path().unwrap_or_default();

    if self.filters.iter().any(|filter| module.starts_with(filter)) {
      return;
    }

    let level = IndigaugeLogLevel::from(metadata.level());

    if !self.levels.contains(&level) {
      return;
    }

    let mut visitor = FieldVisitor::default();
    event.record(&mut visitor);

    if self.event_type_required && visitor.event_type.is_none() {
      return;
    }

    let level_str: &'static str = level.as_str();
    let event_type = visitor.event_type.unwrap_or_else(|| format!("tracing.{}", level_str));
    let event_metadata =
      (!visitor.fields.is_empty()).then(|| serde_json::to_value(&visitor.fields).unwrap_or_default());
    let file = metadata.file().unwrap_or("unknown file");
    let line = metadata.line().unwrap_or_default();

    self
      .sink
      .log(level_str, &event_type, event_metadata, file, line, module);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::api_types::{EventPayload, EventPayloadCtx};

  use serde_json::json;
  use std::sync::{Arc, Mutex};
  use tracing::subscriber::DefaultGuard;
  use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

  struct TestSink {
    events: Mutex<Vec<EventPayload>>,
  }

  impl TestSink {
    fn new() -> Self {
      Self {
        events: Mutex::new(Vec::new()),
      }
    }

    fn take_events(&self) -> Vec<EventPayload> {
      self.events.lock().unwrap().clone()
    }
  }

  impl IndigaugeSink for TestSink {
    fn log(
      &self,
      level: &'static str,
      event_type: &str,
      metadata: Option<serde_json::Value>,
      file: &'static str,
      line: u32,
      module: &'static str,
    ) {
      let module = if module.is_empty() { None } else { Some(module) };
      let context = matches!(level, "warn" | "error").then(|| EventPayloadCtx {
        file: file.to_string(),
        line,
        module,
      });
      let payload = EventPayload {
        level,
        event_type: event_type.to_string(),
        elapsed_ms: 1,
        metadata,
        idempotency_key: None,
        context,
      };
      self.events.lock().unwrap().push(payload);
    }
  }

  /// Creates a subscriber with the IndigaugeLayer installed and returns a guard
  /// so the default subscriber is reset when dropped.
  fn with_indigauge_layer(layer: IndigaugeLayer) -> DefaultGuard {
    let subscriber = tracing_subscriber::registry().with(layer);
    tracing::subscriber::set_default(subscriber)
  }

  #[test]
  fn default_layer_events() {
    let sink = Arc::new(TestSink::new());
    let layer = IndigaugeLayer {
      sink: sink.clone(),
      filters: vec![], // The default filters will filter out test events
      ..Default::default()
    };

    let _guard = with_indigauge_layer(layer);

    tracing::info!("Test default info event type");
    tracing::warn!("Test default warn event type");
    tracing::error!("Test default error event type");
    tracing::info!(message = "Test set event type", event_type = "custom.event");
    tracing::info!(message = "Test set ig", ig = "custom.event");
    tracing::info!(ig = "custom.event");

    let events = sink.take_events();
    assert_eq!(events.len(), 6);

    let expected = [
      ("info", "tracing.info", "Test default info event type", false),
      ("warn", "tracing.warn", "Test default warn event type", true),
      ("error", "tracing.error", "Test default error event type", true),
      ("info", "custom.event", "Test set event type", false),
      ("info", "custom.event", "Test set ig", false),
      ("info", "custom.event", "", false),
    ];

    events
      .iter()
      .zip(expected.iter())
      .for_each(|(event, (level, t, message, has_context))| {
        assert_eq!(event.level, *level);
        assert_eq!(event.event_type, t.to_string());

        if !message.is_empty() {
          assert_eq!(
            event.metadata,
            Some(json!({
              "message": message
            }))
          );
        }

        if *has_context {
          let ctx = event.context.clone().expect("Context");
          assert!(ctx.file.ends_with("tracing.rs"));
          assert_eq!(ctx.module, Some("bevy_mod_indigauge::tracing::tests"));
        }
      });
  }

  #[test]
  fn events_are_filtered_out_by_module() {
    let sink = Arc::new(TestSink::new());
    let layer = IndigaugeLayer {
      sink: sink.clone(),
      ..Default::default()
    };

    let _guard = with_indigauge_layer(layer);

    tracing::info!(message = "Test default info event type");
    tracing::warn!(message = "Test default warn event type");
    tracing::error!(message = "Test default error event type");
    tracing::info!(message = "Test set event type", event_type = "custom.event");
    tracing::info!(message = "Test set ig", ig = "custom.event");

    let events = sink.take_events();
    assert_eq!(events.len(), 0);
  }

  #[test]
  fn events_are_filtered_out_by_level() {
    let sink = Arc::new(TestSink::new());
    let layer = IndigaugeLayer {
      sink: sink.clone(),
      filters: vec![], // The default filters will filter out test events
      levels: vec![IndigaugeLogLevel::Error],
      ..Default::default()
    };

    let _guard = with_indigauge_layer(layer);

    tracing::info!(message = "Test default info event type");
    tracing::warn!(message = "Test default warn event type");
    tracing::error!(message = "Test default error event type");
    tracing::info!(message = "Test set event type", event_type = "custom.event");
    tracing::info!(message = "Test set ig", ig = "custom.event");

    let events = sink.take_events();
    assert_eq!(events.len(), 1);

    assert_eq!(events[0].level, "error");
    assert_eq!(events[0].event_type, "tracing.error");
    assert_eq!(
      events[0].metadata,
      Some(json!({
        "message": "Test default error event type"
      }))
    );
  }

  #[test]
  fn events_are_filtered_out_by_required_event_type() {
    let sink = Arc::new(TestSink::new());
    let layer = IndigaugeLayer {
      sink: sink.clone(),
      filters: vec![], // The default filters will filter out test events
      event_type_required: true,
      ..Default::default()
    };

    let _guard = with_indigauge_layer(layer);

    tracing::info!(message = "Test default info event type");
    tracing::warn!(message = "Test default warn event type");
    tracing::error!(message = "Test default error event type");
    tracing::info!(message = "Test set ig", ig = "custom.event");

    let events = sink.take_events();
    assert_eq!(events.len(), 1);

    assert_eq!(events[0].level, "info");
    assert_eq!(events[0].event_type, "custom.event");
    assert_eq!(
      events[0].metadata,
      Some(json!({
        "message": "Test set ig"
      }))
    );
  }

  #[test]
  fn fields_are_captured_as_json_strings() {
    let sink = Arc::new(TestSink::new());
    let layer = IndigaugeLayer {
      sink: sink.clone(),
      filters: vec![], // The default filters will filter out test events
      ..Default::default()
    };

    let _guard = with_indigauge_layer(layer);

    tracing::info!(foo = 42, bar = "baz", "checking fields");

    let events = sink.take_events();
    assert_eq!(events.len(), 1);

    assert_eq!(
      events[0],
      EventPayload {
        level: "info",
        event_type: "tracing.info".to_string(),
        elapsed_ms: 1,
        metadata: Some(json!({
          "foo": 42,
          "bar": "baz",
          "message": "checking fields"
        })),
        idempotency_key: None,
        context: None,
      }
    );
  }
}
