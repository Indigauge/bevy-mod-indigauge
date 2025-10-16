use std::time::Instant;

use bevy::{
  input::mouse::{MouseScrollUnit, MouseWheel},
  picking::focus::HoverMap,
  prelude::*,
};
use bevy_text_edit::TextEditable;

use crate::{
  SESSION_START_INSTANT,
  api_types::FeedbackPayload,
  primitives::feedback::*,
  resources::{feedback::*, session::SessionApiKey},
  sysparam::BevyIndigauge,
  utils::select,
};

const LINE_HEIGHT: f32 = 21.;

pub fn despawn_feedback_panel(mut commands: Commands, query: Query<Entity, With<FeedbackPanel>>) {
  for entity in &query {
    commands.entity(entity).despawn_recursive();
  }
}

// Submit
pub fn submit_click_system(
  mut commands: Commands,
  q: Query<&Interaction, (With<SubmitButton>, Changed<Interaction>)>,
  q_input: Query<&Text, With<MessageInput>>,
  mut form: ResMut<FeedbackFormState>,
  mut ig: BevyIndigauge,
  session_key: Res<SessionApiKey>,
) {
  for interaction in &q {
    if *interaction != Interaction::Pressed {
      continue;
    }

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

      let screenshot_url: Option<String> = if form.include_screenshot {
        // TODO: koble til faktisk screenshot-pipeline (returner URL eller legg ved opplasting)
        None
      } else {
        None
      };

      let payload = FeedbackPayload {
        message: &msg,
        category: form.category.label().to_lowercase(),
        elapsed_ms,
        question: form.question.as_ref(),
      };

      ig.send_feedback(&session_key, &payload);

      commands.remove_resource::<FeedbackPanelProps>();
    }
  }
}

pub fn toggle_panel_visibility_with_key(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  toggle_button: Res<FeedbackKeyCodeToggle>,
  props: Option<ResMut<FeedbackPanelProps>>,
) {
  if keys.just_pressed(toggle_button.0) {
    if let Some(mut props) = props {
      props.visible = !props.visible;
    } else {
      commands.insert_resource(FeedbackPanelProps::visible());
    }
  }
}

// Synk display med visible
pub fn panel_visibility_sync(props: Res<FeedbackPanelProps>, mut q: Query<&mut Node, With<FeedbackPanel>>) {
  if let Ok(mut node) = q.get_single_mut() {
    node.display = select(Display::Flex, Display::None, props.visible);
  }
}

type HoverAndClickInteractionQuery<'a, 'w, 's> = Query<
  'w,
  's,
  (
    &'a Interaction,
    Entity,
    &'a mut BackgroundColor,
    &'a mut BorderColor,
    Option<&'a ButtonHoverStyle>,
    Option<&'a ButtonPressedStyle>,
    Option<&'a OriginalButtonStyles>,
    Has<HoldPressed>,
    Has<Active>,
  ),
  (Changed<Interaction>, Or<(With<ButtonHoverStyle>, With<ButtonPressedStyle>)>),
>;

// Handle hover and click states
pub fn handle_hover_and_click_styles(mut commands: Commands, mut q: HoverAndClickInteractionQuery) {
  q.iter_mut().for_each(
    |(interaction, entity, mut bg_color, mut border_color, bhs, bps, obs, hold_after_press, is_active)| {
      match *interaction {
        Interaction::Hovered => {
          commands.entity(entity).insert_if_new(OriginalButtonStyles {
            background: bg_color.0,
            border: border_color.0,
          });

          if !is_active && let Some(hover_style) = bhs {
            bg_color.0 = hover_style.background;
            border_color.0 = hover_style.border;
          }
        },
        Interaction::Pressed => {
          if let Some(pressed_style) = bps {
            bg_color.0 = pressed_style.background;
            border_color.0 = pressed_style.border;
          }

          if hold_after_press {
            commands.entity(entity).insert(Active);
          }
        },
        _ => {
          if !is_active && let Some(original_styles) = obs {
            bg_color.0 = original_styles.background;
            border_color.0 = original_styles.border;
          }
        },
      }
    },
  );
}

// Toggle dropdown
pub fn category_toggle_system(
  mut form: ResMut<FeedbackFormState>,
  q: Query<&Interaction, (With<CategoryButton>, Changed<Interaction>)>,
) {
  for interaction in &q {
    if *interaction == Interaction::Pressed {
      form.dropdown_open = !form.dropdown_open;
    }
  }
}

type CategoryItemInteractionQuery<'a, 'w, 's> =
  Query<'w, 's, (&'a Interaction, &'a CategoryItem), (With<Button>, Changed<Interaction>)>;
// Plukke kategori + oppdatere knapptittel
pub fn category_pick_system(
  mut form: ResMut<FeedbackFormState>,
  q_items: CategoryItemInteractionQuery,
  mut q_btn_text_root: Query<&mut TextSpan, With<CategoryButtonText>>,
) {
  for (interaction, CategoryItem(cat)) in &q_items {
    if *interaction == Interaction::Pressed {
      form.category = *cat;
      form.dropdown_open = false;

      // Oppdater knappetekst
      if let Ok(mut root) = q_btn_text_root.get_single_mut() {
        **root = cat.label().to_string();
      }
    }
  }
}

// Synk dropdown synlighet
pub fn dropdown_visibility_sync(form: Res<FeedbackFormState>, mut q: Query<&mut Node, With<CategoryList>>) {
  if !form.is_changed() {
    return;
  }
  if let Ok(mut n) = q.get_single_mut() {
    n.display = select(Display::Flex, Display::None, form.dropdown_open);
  }
}

type ScreenshotToggleInteractionQuery<'a, 'w, 's> =
  Query<'w, 's, (&'a Interaction, &'a mut BackgroundColor), (With<ScreenshotToggle>, Changed<Interaction>)>;
// Screenshot toggle
pub fn screenshot_toggle_click_system(
  styles: Res<FeedbackPanelStyles>,
  mut form: ResMut<FeedbackFormState>,
  mut q: ScreenshotToggleInteractionQuery,
  mut q_text_root: Query<(&mut TextSpan, &mut TextColor), With<ScreenshotToggleText>>,
) {
  q.iter_mut().for_each(|(interaction, mut bg_color)| {
    if *interaction == Interaction::Pressed {
      form.include_screenshot = !form.include_screenshot;
      bg_color.0 = select(styles.accent, styles.surface, form.include_screenshot);
      if let Ok((mut root, mut color)) = q_text_root.get_single_mut() {
        **root = format!("Include screenshot: {}", select("Yes", "No", form.include_screenshot));
        color.0 = select(styles.text_primary, styles.text_secondary, form.include_screenshot);
      }
    }
  });
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
  mut mouse_wheel_events: EventReader<MouseWheel>,
  hover_map: Res<HoverMap>,
  mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
  for mouse_wheel_event in mouse_wheel_events.read() {
    let (dx, dy) = match mouse_wheel_event.unit {
      MouseScrollUnit::Line => (mouse_wheel_event.x * LINE_HEIGHT, mouse_wheel_event.y * LINE_HEIGHT),
      MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
    };

    for (_pointer, pointer_map) in hover_map.iter() {
      for (entity, _hit) in pointer_map.iter() {
        if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
          scroll_position.offset_x -= dx;
          scroll_position.offset_y -= dy;
        }
      }
    }
  }
}

pub fn spawn_feedback_ui(
  mut commands: Commands,
  styles: Res<FeedbackPanelStyles>,
  props: Res<FeedbackPanelProps>,
  mut form: ResMut<FeedbackFormState>,
  feedback_panel_query: Query<Entity, With<FeedbackPanel>>,
) {
  if let Ok(root_entity) = feedback_panel_query.get_single() {
    commands.entity(root_entity).despawn_recursive();
  }

  *form = FeedbackFormState::default();

  if let Some(category) = &props.category
    && let Some(question) = &props.question
  {
    form.category = *category;
    form.question = Some(question.clone());
  }

  // Root overlay
  commands
    .spawn((
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
      },
      BackgroundColor(Color::NONE),
      FeedbackPanel,
    ))
    .with_children(|root| {
      // Panel/card
      root
        .spawn((
          Node {
            width: Val::Px(420.0),
            min_height: Val::Px(420.0),
            padding: UiRect::axes(Val::Px(48.0), Val::Px(32.0)),
            border: UiRect::all(Val::Px(2.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
          },
          panel(styles.background, styles.border),
        ))
        .with_children(|child_panel| {
          // Title
          child_panel
            .spawn((Text::default(), Node::default()))
            .with_children(|t| {
              t.spawn((TextSpan::new("Send feedback"), TextFont::from_font_size(22.), TextColor(styles.text_primary)));
            });

          if let Some(question) = &props.question {
            child_panel
              .spawn((Text::default(), Node::default()))
              .with_children(|t| {
                t.spawn((
                  Text::new(question),
                  QuestionTextRoot,
                  TextFont::from_font_size(18.),
                  TextColor(styles.text_secondary),
                ));
              });
          } else {
            // Category
            child_panel
              .spawn((
                Node {
                  width: Val::Percent(100.0),
                  height: Val::Auto,
                  justify_content: JustifyContent::SpaceBetween,
                  align_items: AlignItems::Center,
                  ..default()
                },
                BackgroundColor(Color::NONE),
              ))
              .with_children(|row| {
                // Category button
                row
                  .spawn((
                    Node {
                      width: Val::Percent(100.0),
                      border: UiRect::all(Val::Px(3.0)),
                      padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                      ..default()
                    },
                    CategoryButton,
                    button(styles.surface, styles.border),
                  ))
                  .with_children(|b| {
                    b.spawn((Text::new("Category: "), TextFont::from_font_size(16.), TextColor(styles.text_primary)));
                    b.spawn((Text::default(), Node::default())).with_children(|t| {
                      t.spawn((
                        TextSpan::new(FeedbackCategory::General.label()),
                        CategoryButtonText,
                        TextFont::from_font_size(16.),
                        TextColor(styles.text_primary),
                      ));
                    });
                  });
              });

            // Dropdown-list (hidden as default)
            child_panel
              .spawn((
                Node {
                  width: Val::Px(318.0),
                  flex_direction: FlexDirection::Row,
                  flex_wrap: FlexWrap::Wrap,
                  justify_content: JustifyContent::SpaceBetween,
                  row_gap: Val::Px(4.0),
                  padding: UiRect::all(Val::Px(8.0)),
                  border: UiRect::all(Val::Px(1.0)),
                  display: Display::None,
                  position_type: PositionType::Absolute,
                  top: Val::Px(110.0),
                  left: Val::Px(49.0),
                  ..default()
                },
                BackgroundColor(styles.background),
                BorderColor(styles.border),
                BorderRadius::bottom(Val::Px(8.)),
                ZIndex(10),
                CategoryList,
              ))
              .with_children(|list| {
                for cat in FeedbackCategory::ALL {
                  list
                    .spawn((
                      Node {
                        width: Val::Percent(48.0),
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                      },
                      CategoryItem(*cat),
                      button(styles.surface, styles.border),
                    ))
                    .with_children(|b| {
                      b.spawn((Text::default(), Node::default())).with_children(|t| {
                        t.spawn((
                          TextSpan::new(cat.label()),
                          TextFont::from_font_size(14.),
                          TextColor(styles.text_primary),
                        ));
                      });
                    });
                }
              });
          }

          // Text-input area
          child_panel
            .spawn((Node {
              width: Val::Percent(100.0),
              min_height: Val::Px(180.0),
              ..default()
            },))
            .with_children(|area| {
              area
                .spawn((
                  Node {
                    width: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                  },
                  MessageInput,
                  panel(styles.surface, styles.border),
                ))
                .with_children(|field| {
                  field.spawn((
                    Node {
                      width: Val::Percent(100.0),
                      height: Val::Percent(100.0),
                      ..default()
                    },
                    Text::new(""),
                    TextFont::from_font_size(16.),
                    TextColor(styles.text_primary),
                    MessageInput,
                    TextEditable {
                      filter_in: vec!["[a-zA-Z0-9 .,;:!?()\"'-]".into(), " ".into()],
                      placeholder: "Provide feedback message here".to_string(),
                      ..Default::default()
                    },
                  ));
                });
            });

          if props.allow_screenshot {
            // Screenshot toggle
            child_panel
              .spawn((
                Node {
                  width: Val::Percent(100.0),
                  justify_content: JustifyContent::SpaceBetween,
                  align_items: AlignItems::Center,
                  ..default()
                },
                BackgroundColor(Color::NONE),
              ))
              .with_children(|row| {
                // Screenshot toggle
                row
                  .spawn((
                    Node {
                      border: UiRect::all(Val::Px(2.0)),
                      padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                      ..default()
                    },
                    ScreenshotToggle,
                    button(styles.surface, styles.border),
                  ))
                  .with_children(|b| {
                    b.spawn((Text::default(), Node::default())).with_children(|t| {
                      t.spawn((
                        TextSpan::new("Include screenshot: No"),
                        ScreenshotToggleText,
                        TextFont::from_font_size(14.),
                        TextColor(styles.secondary),
                      ));
                    });
                  });
              });
          }

          // Submit and cancel buttons
          child_panel
            .spawn((
              Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::top(Val::Px(15.)),
                ..default()
              },
              BackgroundColor(Color::NONE),
            ))
            .with_children(|row| {
              // Cancel
              row
                .spawn((
                  Node {
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    ..default()
                  },
                  CancelButton,
                  button(styles.surface, styles.border),
                ))
                .with_children(|b| {
                  b.spawn((Text::default(), Node::default())).with_children(|t| {
                    t.spawn((TextSpan::new("Cancel"), TextFont::from_font_size(16.), TextColor(styles.text_secondary)));
                  });
                });

              // Submit
              row
                .spawn((
                  Button,
                  Node {
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    ..default()
                  },
                  SubmitButton,
                  ButtonHoverStyle {
                    background: styles.primary_hover,
                    border: styles.border.with_alpha(0.5),
                  },
                  ButtonPressedStyle {
                    background: styles.primary_hover.with_alpha(0.5),
                    border: styles.border.with_alpha(0.2),
                  },
                  panel(styles.primary, styles.primary_hover),
                ))
                .with_children(|b| {
                  b.spawn((Text::default(), Node::default())).with_children(|t| {
                    t.spawn((TextSpan::new("Send"), TextFont::from_font_size(16.), TextColor(styles.text_primary)));
                  });
                });
            });
        });
    });
}
