use std::fs;
use std::time::Instant;

use bevy::ecs::bundle::Bundle;
use bevy::ecs::observer::Trigger;
use bevy::ecs::system::{IntoObserverSystem, Res, ResMut, SystemParam};
use bevy::log::{error, info};
use bevy_mod_reqwest::reqwest::{Error as ReqwestError, Request};
use bevy_mod_reqwest::{BevyReqwest, ReqwestErrorEvent, ReqwestResponseEvent};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::api_types::{BatchEventPayload, FeedbackPayload};
use crate::resources::events::BufferedEvents;
use crate::resources::{IndigaugeConfig, IndigaugeMode};
use crate::{IndigaugeLogLevel, LastSentRequestInstant};

#[derive(SystemParam)]
pub struct BevyIndigauge<'w, 's> {
  pub reqwest_client: BevyReqwest<'w, 's>,
  pub config: Res<'w, IndigaugeConfig>,
  pub buffered_events: ResMut<'w, BufferedEvents>,
  pub last_sent_request: ResMut<'w, LastSentRequestInstant>,
  pub log_level: Res<'w, IndigaugeLogLevel>,
  pub mode: Res<'w, IndigaugeMode>,
}

impl<'w, 's> BevyIndigauge<'w, 's> {
  pub fn build_request<S>(&self, url: &str, ig_key: &str, payload: &S) -> Result<Request, ReqwestError>
  where
    S: Serialize,
  {
    let url = format!("{}/v1/{}", &self.config.api_base, url);

    self
      .reqwest_client
      .post(url)
      .timeout(self.config.request_timeout)
      .header("Content-Type", "application/json")
      .header("X-Indigauge-Key", ig_key)
      .json(payload)
      .build()
  }

  pub fn send_feedback_screenshot(&mut self, api_key: &str, feedback_id: &str, image_data: Vec<u8>) {
    match *self.mode {
      IndigaugeMode::Live => {
        let url = format!("feedback/{}/screenshot", feedback_id);
        let url = format!("{}/v1/{}", &self.config.api_base, url);

        let request = self
          .reqwest_client
          .post(url)
          .timeout(self.config.request_timeout)
          .header("Content-Type", "image/png")
          .header("X-Indigauge-Key", api_key)
          .body(image_data)
          .build();

        if let Ok(request) = request {
          self.last_sent_request.instant = Instant::now();
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: Trigger<ReqwestResponseEvent>, log_level: Res<IndigaugeLogLevel>| {
              if trigger.status().is_success() {
                if *log_level <= IndigaugeLogLevel::Info {
                  info!(message = "Sent feedback screenshot");
                }
              } else if *log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send feedback screenshot");
              }
            })
            .on_error(|trigger: Trigger<ReqwestErrorEvent>, log_level: Res<IndigaugeLogLevel>| {
              if *log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send feedback", error = ?trigger.event().0);
              }
            });
        }
      },
      IndigaugeMode::Dev => {
        if *self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: Sent feedback screenshot");
        }
      },
      _ => {},
    }
  }

  pub fn send_feedback<RB, RM, OR>(&mut self, api_key: &str, payload: &FeedbackPayload, on_response: OR)
  where
    RB: Bundle,
    OR: IntoObserverSystem<ReqwestResponseEvent, RB, RM>,
  {
    match *self.mode {
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_request("feedback", api_key, payload) {
          self.last_sent_request.instant = Instant::now();
          self.reqwest_client.send(request).on_response(on_response).on_error(
            |trigger: Trigger<ReqwestErrorEvent>, log_level: Res<IndigaugeLogLevel>| {
              if *log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send feedback", error = ?trigger.event().0);
              }
            },
          );
        }
      },
      IndigaugeMode::Dev => {
        if *self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: Sent feedback", feedback = ?payload);
        }
      },
      _ => {},
    }
  }

  pub fn flush_events(&mut self, api_key: &str) -> usize {
    let event_len = self.buffered_events.events.len();
    if event_len == 0 {
      return 0;
    }

    let events = BatchEventPayload {
      events: self
        .buffered_events
        .events
        .drain(..(event_len.min(self.config.batch_size)))
        .map(|event| event.into_inner())
        .collect::<Vec<_>>(),
    };

    match *self.mode {
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_request("events/batch", api_key, &events) {
          self.last_sent_request.instant = Instant::now();
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: Trigger<ReqwestResponseEvent>| {
              dbg!(trigger.event().body());
            })
            .on_error(|trigger: Trigger<ReqwestErrorEvent>, log_level: Res<IndigaugeLogLevel>| {
              if *log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send event batch", error = ?trigger.event().0);
              }
            });
        }
      },
      IndigaugeMode::Dev => {
        if *self.log_level <= IndigaugeLogLevel::Info {
          info!(message = "DEVMODE: sending event batch", count = events.events.len());
        }
      },
      _ => {},
    }

    events.events.len()
  }

  pub fn send_heartbeat(&mut self, api_key: &str) {
    match *self.mode {
      IndigaugeMode::Live => {
        if let Ok(request) = self.build_request("sessions/heartbeat", api_key, &json!({})) {
          self.last_sent_request.instant = Instant::now();
          self
            .reqwest_client
            .send(request)
            .on_response(|trigger: Trigger<ReqwestResponseEvent>| {
              dbg!("heartbeat response: {:?}", trigger.event().body());
            })
            .on_error(|trigger: Trigger<ReqwestErrorEvent>, log_level: Res<IndigaugeLogLevel>| {
              if *log_level <= IndigaugeLogLevel::Error {
                error!(message = "Failed to send session heartbeat", error = ?trigger.event().0);
              }
            });
        }
      },
      IndigaugeMode::Dev => {
        if *self.log_level <= IndigaugeLogLevel::Info {
          info!("DEVMODE: heartbeat");
        }
      },
      _ => {},
    }
  }

  pub fn get_or_init_player_id(&self) -> String {
    let game_folder_path = dirs::preference_dir().map(|dir| dir.join(&self.config.game_name));

    if let Some(game_folder_path) = game_folder_path {
      let player_id_file_path = game_folder_path.join("player_id.txt");

      if let Ok(player_id) = fs::read_to_string(&player_id_file_path) {
        player_id
      } else {
        let new_player_id = Uuid::new_v4().to_string();
        let _ = fs::create_dir_all(&game_folder_path);
        let _ = fs::write(&player_id_file_path, &new_player_id);
        new_player_id
      }
    } else {
      Uuid::new_v4().to_string()
    }
  }
}
