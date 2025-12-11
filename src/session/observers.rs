use std::{env::consts::OS, time::Instant};

use bevy::{diagnostic::SystemInfo, prelude::*, render::renderer::RenderAdapterInfo, state::state::FreelyMutableState};
use bevy_mod_reqwest::{ReqwestErrorEvent, ReqwestResponseEvent};

use crate::{
  api_types::{ApiResponse, StartSessionPayload, StartSessionResponse},
  config::IndigaugeConfig,
  config::IndigaugeMode,
  plugin::GLOBAL_TX,
  prelude::*,
  session::SESSION_START_INSTANT,
  session::resources::SessionApiKey,
  session::utils::{bucket_cores, bucket_ram_gb, coarsen_cpu_name},
  utils::BevyIndigauge,
};

pub fn switch_state_after_session_init<S>(state: S) -> impl FnMut(Trigger<IndigaugeInitDoneEvent>, ResMut<NextState<S>>)
where
  S: FreelyMutableState + Copy,
{
  move |_trigger, mut next_state| {
    next_state.set(state);
  }
}

pub fn observe_start_session_event(
  event: Trigger<StartSessionEvent>,
  mut ig: BevyIndigauge,
  mut cmd: Commands,
  sys_info: Res<SystemInfo>,
  render_info: Res<RenderAdapterInfo>,
) {
  if SESSION_START_INSTANT.get().is_some() {
    if *ig.log_level <= IndigaugeLogLevel::Warn {
      warn!("Session already started");
    }
    cmd.trigger(IndigaugeInitDoneEvent::Skipped("Session already started".to_string()));
    return;
  }

  if GLOBAL_TX.get().is_none() {
    cmd.trigger(IndigaugeInitDoneEvent::UnexpectedFailure("Global transaction not initialized".to_string()));
    return;
  }

  match *ig.mode {
    IndigaugeMode::Dev => {
      let dev_response = StartSessionResponse::dev();
      start_session(&mut cmd, dev_response, &ig.log_level, &ig.mode, &ig.config);
      return;
    },
    IndigaugeMode::Disabled => {
      cmd.trigger(IndigaugeInitDoneEvent::Skipped("Indigauge disabled".to_string()));
      return;
    },
    _ => {},
  }

  #[cfg(not(target_family = "wasm"))]
  let player_id = Some(ig.get_or_init_player_id());

  #[cfg(target_family = "wasm")]
  let player_id = None::<String>;

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
    player_id: player_id.as_ref(),
    platform: event.platform.as_ref(),
    os: Some(OS),
    cpu_family: cpu_family.as_ref(),
    cores,
    memory,
    gpu: Some(&render_info.name),
  };

  let reqwest_client = ig.build_post_request("sessions/start", &ig.config.public_key, &payload);

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
  log_level: Res<IndigaugeLogLevel>,
  mode: Res<IndigaugeMode>,
) {
  let Ok(response) = trigger.event().deserialize_json::<ApiResponse<StartSessionResponse>>() else {
    if *log_level <= IndigaugeLogLevel::Error {
      error!("Failed to deserialize response");
    }
    commands.trigger(IndigaugeInitDoneEvent::UnexpectedFailure("Failed to deserialize response".to_string()));
    return;
  };

  match response {
    ApiResponse::Ok(response) => {
      start_session(&mut commands, response, &log_level, &mode, &ig_config);
    },
    ApiResponse::Err(error_body) => {
      if *log_level <= IndigaugeLogLevel::Error {
        error!(message = "Failed to start session", error_code = error_body.code, error_message = error_body.message);
      }
      commands.trigger(IndigaugeInitDoneEvent::Failure("Failed to start session".to_string()));
    },
  }
}

pub fn on_start_session_error(
  trigger: Trigger<ReqwestErrorEvent>,
  mut commands: Commands,
  log_level: Res<IndigaugeLogLevel>,
) {
  if *log_level <= IndigaugeLogLevel::Error {
    error!(message = "Create session post request failed", error = %trigger.event().0);
  }
  commands.trigger(IndigaugeInitDoneEvent::Failure("Create session post request failed".to_string()));
}

#[allow(unused_variables)]
fn start_session(
  commands: &mut Commands,
  response: StartSessionResponse,
  log_level: &IndigaugeLogLevel,
  mode: &IndigaugeMode,
  config: &IndigaugeConfig,
) {
  let start_instant = Instant::now();
  if let Err(set_start_instance_err) = SESSION_START_INSTANT.set(start_instant) {
    if *log_level <= IndigaugeLogLevel::Error {
      error!(message = "Failed to set session start instant", error = ?set_start_instance_err);
    }
    commands.trigger(IndigaugeInitDoneEvent::Failure("Failed to set session start instant".to_string()));
    return;
  }

  if *log_level <= IndigaugeLogLevel::Info {
    match *mode {
      IndigaugeMode::Live => {
        info!(message = "Indigauge session started");
      },
      IndigaugeMode::Dev => {
        info!(message = "DEVMODE: Indigauge session started");
      },
      IndigaugeMode::Disabled => {},
    }
  }

  let key = response.session_token.clone();

  #[cfg(all(feature = "panic_handler", not(target_family = "wasm")))]
  {
    use crate::session::utils::panic_handler;

    let host_origin = config.api_base.clone();
    std::panic::set_hook(Box::new(panic_handler(host_origin, key.clone())));
  }

  commands.insert_resource(SessionApiKey::new(key));
  commands.trigger(IndigaugeInitDoneEvent::Success);
}
