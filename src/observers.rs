use std::{env::consts::OS, time::Instant};

use bevy::{diagnostic::SystemInfo, prelude::*};
use bevy_mod_reqwest::{ReqwestErrorEvent, ReqwestResponseEvent};

use crate::{
  GLOBAL_TX, IndigaugeInitDoneEvent, SESSION_START_INSTANT, SessionApiKey, StartSessionEvent,
  api_types::{ApiResponse, StartSessionPayload, StartSessionResponse},
  resources::IndigaugeConfig,
  sysparam::BevyIndigauge,
  utils::{bucket_cores, bucket_ram_gb, coarsen_cpu_name},
};

pub fn observe_start_session_event(
  event: Trigger<StartSessionEvent>,
  mut ig: BevyIndigauge,
  mut cmd: Commands,
  sys_info: Res<SystemInfo>,
) {
  if SESSION_START_INSTANT.get().is_some() {
    warn!("Session already started");
    cmd.trigger(IndigaugeInitDoneEvent::Skipped("Session already started".to_string()));
    return;
  }

  if GLOBAL_TX.get().is_none() {
    cmd.trigger(IndigaugeInitDoneEvent::UnexpectedFailure("Global transaction not initialized".to_string()));
    return;
  }

  let player_id = ig.get_or_init_player_id();

  let event = event.event();
  let cores = sys_info.core_count.parse().map(bucket_cores).ok();
  let memory = sys_info
    .memory
    .split('.')
    .collect::<Vec<_>>()
    .first()
    .and_then(|m| m.parse().map(bucket_ram_gb).ok());
  let cpu_family = coarsen_cpu_name(&sys_info.cpu);

  let payload = StartSessionPayload {
    client_version: &ig.config.game_version,
    player_id: Some(&player_id),
    platform: event.platform.as_ref(),
    os: Some(OS),
    locale: event.locale.as_ref(),
    cpu_family: cpu_family.as_ref(),
    cores,
    memory,
  };

  let reqwest_client = ig.build_request("sessions/start", &ig.config.public_key, &payload);

  match reqwest_client {
    Ok(reqwest_client) => {
      ig.reqwest_client
        .send(reqwest_client)
        .on_response(on_start_session_response)
        .on_error(on_start_session_error);
    },
    Err(err) => {
      error!("Failed to create session post client: {}", err);
      cmd.trigger(IndigaugeInitDoneEvent::Failure("Failed to create session post client".to_string()));
    },
  }
}

pub fn on_start_session_response(
  trigger: Trigger<ReqwestResponseEvent>,
  mut commands: Commands,
  ig_config: Res<IndigaugeConfig>,
) {
  let Ok(response) = trigger.event().deserialize_json::<ApiResponse<StartSessionResponse>>() else {
    error!("Failed to deserialize response");
    commands.trigger(IndigaugeInitDoneEvent::UnexpectedFailure("Failed to deserialize response".to_string()));
    return;
  };

  match response {
    ApiResponse::Ok(response) => {
      let start_instant = Instant::now();
      if let Err(set_start_instance_err) = SESSION_START_INSTANT.set(start_instant) {
        error!(message = "Failed to set session start instant", error = ?set_start_instance_err);
        commands.trigger(IndigaugeInitDoneEvent::Failure("Failed to set session start instant".to_string()));
        return;
      }

      info!(message = "Indigauge session started", start_instant = ?start_instant);
      let key = response.session_token.clone();

      #[cfg(feature = "panic_handler")]
      {
        use crate::utils::panic_handler;

        let host_origin = ig_config.api_base.clone();
        std::panic::set_hook(Box::new(panic_handler(host_origin, key.clone())));
      }

      commands.insert_resource(SessionApiKey::new(key));
      commands.trigger(IndigaugeInitDoneEvent::Success);
    },
    ApiResponse::Err(error_body) => {
      error!(message = "Failed to start session", error_code = error_body.code, error_message = error_body.message);
      commands.trigger(IndigaugeInitDoneEvent::Failure("Failed to start session".to_string()));
    },
  }
}

pub fn on_start_session_error(trigger: Trigger<ReqwestErrorEvent>, mut commands: Commands) {
  error!(message = "Create session post request failed", error = ?trigger.event().0);
  commands.trigger(IndigaugeInitDoneEvent::Failure("Create session post request failed".to_string()));
}
