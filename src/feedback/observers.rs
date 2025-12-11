use std::{ops::Deref, time::Instant};

use bevy::{
  prelude::*,
  render::view::screenshot::{Screenshot, ScreenshotCaptured},
  state::state::FreelyMutableState,
};
use bevy_mod_reqwest::ReqwestResponseEvent;
use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};

use crate::{
  api_types::{FeedbackPayload, IdResponse},
  feedback::components::{CategoryButtonText, CategoryItem, FeedbackPanel, MessageInput, ScreenshotToggleText},
  feedback::resources::{FeedbackFormState, TakeScreenshot},
  prelude::*,
  session::SESSION_START_INSTANT,
  session::resources::SessionApiKey,
  utils::BevyIndigauge,
  utils::select,
};

pub fn switch_state_on_feedback_spawn<S>(state: S) -> impl FnMut(Trigger<OnAdd, FeedbackPanel>, ResMut<NextState<S>>)
where
  S: FreelyMutableState + Copy,
{
  move |_trigger, mut next_state| {
    next_state.set(state);
  }
}

pub fn switch_state_on_feedback_despawn<S>(
  state: S,
) -> impl FnMut(Trigger<OnRemove, FeedbackPanel>, ResMut<NextState<S>>)
where
  S: FreelyMutableState + Copy,
{
  move |_trigger, mut next_state| {
    next_state.set(state);
  }
}

pub fn observe_category_dropdown_click(_trigger: Trigger<Pointer<Click>>, mut form: ResMut<FeedbackFormState>) {
  form.dropdown_open = !form.dropdown_open;
}

pub fn observe_category_item_click(
  trigger: Trigger<Pointer<Click>>,
  mut form: ResMut<FeedbackFormState>,
  category_item_query: Query<&CategoryItem>,
  mut q_btn_text_root: Query<&mut TextSpan, With<CategoryButtonText>>,
) {
  let Ok(CategoryItem(category)) = category_item_query.get(trigger.entity()) else {
    return;
  };

  form.category = *category;
  form.dropdown_open = false;

  // Update button text
  if let Ok(mut root) = q_btn_text_root.get_single_mut() {
    **root = category.label().to_string();
  }
}

pub fn observe_screenshot_toggle_click(
  trigger: Trigger<Pointer<Click>>,
  styles: Res<FeedbackPanelStyles>,
  mut form: ResMut<FeedbackFormState>,
  mut q: Query<&mut BackgroundColor>,
  mut q_text_root: Query<(&mut TextSpan, &mut TextColor), With<ScreenshotToggleText>>,
) {
  let Ok(mut bg_color) = q.get_mut(trigger.entity()) else {
    return;
  };

  form.include_screenshot = !form.include_screenshot;

  bg_color.0 = select(styles.accent, styles.surface, form.include_screenshot);

  if let Ok((mut root, mut color)) = q_text_root.get_single_mut() {
    **root = select("Yes", "No", form.include_screenshot).to_string();
    color.0 = select(styles.text_primary, styles.text_secondary, form.include_screenshot);
  }
}

pub fn observe_cancel_click(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
  commands.remove_resource::<FeedbackPanelProps>();
}

pub fn observe_submit_click(
  _trigger: Trigger<Pointer<Click>>,
  mut commands: Commands,
  q_input: Query<&Text, With<MessageInput>>,
  mut form: ResMut<FeedbackFormState>,
  mut ig: BevyIndigauge,
  session_key: Res<SessionApiKey>,
) {
  if let Some(start_instant) = SESSION_START_INSTANT.get() {
    let elapsed_ms = Instant::now().duration_since(*start_instant).as_millis();

    let msg = q_input
      .get_single()
      .map(|s| s.to_string())
      .unwrap_or_default()
      .replace("\r\n", "\n")
      .replace('\r', "\n")
      .replace("  ", " ")
      .trim()
      .to_string();

    if msg.len().lt(&2) {
      form.error = Some("Feedback cannot be less than 2 characters".to_string());
      return;
    }

    if form.include_screenshot {
      commands.insert_resource(TakeScreenshot);
    }

    let payload = FeedbackPayload {
      message: &msg,
      category: form.category.label().to_lowercase(),
      elapsed_ms,
      question: form.question.as_ref(),
    };

    ig.send_feedback(&session_key, &payload, maybe_take_screenshot);

    commands.remove_resource::<FeedbackPanelProps>();
  }
}

fn maybe_take_screenshot(
  trigger: Trigger<ReqwestResponseEvent>,
  mut commands: Commands,
  take_screenshot: Option<Res<TakeScreenshot>>,
) {
  if take_screenshot.is_some()
    && let Ok(feedback_id) = trigger.event().deserialize_json::<IdResponse>()
  {
    commands.remove_resource::<TakeScreenshot>();
    commands.spawn(Screenshot::primary_window()).observe(
      move |trigger: Trigger<ScreenshotCaptured>, mut ig: BevyIndigauge, api_key: Res<SessionApiKey>| {
        let img = trigger.event().deref().clone();

        match img.try_into_dynamic() {
          Ok(dyn_img) => {
            let data = dyn_img.to_rgb8().to_vec();
            let mut png = Vec::new();
            let enc = PngEncoder::new(&mut png);

            if enc
              .write_image(&data, dyn_img.width(), dyn_img.height(), ColorType::Rgb8)
              .is_ok()
            {
              ig.send_feedback_screenshot(&api_key, &feedback_id, png);
            } else if *ig.log_level <= IndigaugeLogLevel::Error {
              error!(message = "Failed to encode screenshot as PNG");
            }
          },
          Err(error) => {
            if *ig.log_level <= IndigaugeLogLevel::Error {
              error!(message = "Failed to convert screenshot into dynamic image", ?error);
            }
          },
        }
      },
    );
  }
}
