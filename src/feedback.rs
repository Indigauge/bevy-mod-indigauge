// feedback_ui.rs
use bevy::{
  input::{
    keyboard::{Key, KeyCode, KeyboardInput},
    mouse::{MouseScrollUnit, MouseWheel},
  },
  picking::focus::HoverMap,
  prelude::*,
};
use serde_json::json;

use crate::feedback::{
  resources::{FeedbackFormState, FeedbackPanelStyles},
  ui_elements::*,
};

pub mod resources;
pub mod ui_elements;

pub use resources::FeedbackPanelProps;

const LINE_HEIGHT: f32 = 21.;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum FeedbackCategory {
  #[default]
  General,
  UI,
  Gameplay,
  Performance,
  Bugs,
  Other,
}

impl FeedbackCategory {
  pub const ALL: &'static [FeedbackCategory] = &[
    FeedbackCategory::General,
    FeedbackCategory::UI,
    FeedbackCategory::Gameplay,
    FeedbackCategory::Performance,
    FeedbackCategory::Bugs,
    FeedbackCategory::Other,
  ];

  pub fn label(&self) -> &'static str {
    match self {
      FeedbackCategory::General => "General",
      FeedbackCategory::UI => "UI",
      FeedbackCategory::Gameplay => "Gameplay",
      FeedbackCategory::Performance => "Performance",
      FeedbackCategory::Bugs => "Bugs",
      FeedbackCategory::Other => "Other",
    }
  }
}

/* ========= Plugin ========= */

pub struct FeedbackUiPlugin;
impl Plugin for FeedbackUiPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<FeedbackFormState>()
      .init_resource::<FeedbackPanelProps>()
      .init_resource::<FeedbackPanelStyles>()
      .add_systems(Startup, spawn_feedback_ui)
      .add_systems(
        Update,
        (
          toggle_panel_visibility_with_key,
          panel_visibility_sync,
          input_focus_via_interaction,
          input_defocus_when_other_buttons_pressed,
          input_type_chars,
          refresh_input_text,
          rating_click_system,
          category_toggle_system,
          category_pick_system,
          dropdown_visibility_sync,
          screenshot_toggle_click_system,
          submit_click_system,
          update_scroll_position,
          handle_hover_and_click_styles,
        ),
      );
  }
}

/* ========= UI build ========= */

fn spawn_feedback_ui(mut commands: Commands, asset_server: Res<AssetServer>, styles: Res<FeedbackPanelStyles>) {
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
    .observe(
      |t: Trigger<Pointer<Click>>,
       mut cmd: Commands,
       message_input_query: Query<Entity, With<MessageInput>>,
       styles: Res<FeedbackPanelStyles>| {
        if message_input_query.get(t.target).is_ok() {
          return;
        }

        if let Ok(entity) = message_input_query.get_single() {
          cmd
            .entity(entity)
            .insert(button(styles.surface, styles.border))
            .remove::<Active>();
        }
      },
    )
    .with_children(|root| {
      // Panel/kort
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
          // Tittel
          child_panel
            .spawn((Text::default(), Node::default()))
            .with_children(|t| {
              t.spawn((TextSpan::new("Send feedback"), TextFont::from_font_size(22.), TextColor(styles.text_primary)));
            });

          // Rad: kategori + rating
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
              // Kategori-knapp
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

              // Rating-stjerner
              // row
              //   .spawn((
              //     Node {
              //       column_gap: Val::Px(4.0),
              //       ..default()
              //     },
              //     BackgroundColor(Color::NONE),
              //   ))
              //   .with_children(|stars| {
              //     for i in 1..=5 {
              //       stars
              //         .spawn((
              //           Button,
              //           Node {
              //             width: Val::Px(28.0),
              //             height: Val::Px(28.0),
              //             align_items: AlignItems::Center,
              //             justify_content: JustifyContent::Center,
              //             ..default()
              //           },
              //           BackgroundColor(Color::srgb(0.16, 0.16, 0.20)),
              //           RatingStar(i),
              //         ))
              //         .with_children(|b| {
              //           b.spawn((
              //             Text::default(),
              //             TextFont {
              //               font: font.clone(),
              //               font_size: 18.0,
              //               ..default()
              //             },
              //             TextColor(Color::WHITE),
              //             Node::default(),
              //           ))
              //           .with_children(|t| {
              //             t.spawn(TextSpan::new("★"));
              //           });
              //         });
              //     }
              //   });
            });

          // Dropdown-liste (skjult til å begynne med)
          child_panel
            .spawn((
              Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                display: Display::None,
                ..default()
              },
              BorderColor(styles.surface),
              CategoryList,
            ))
            .with_children(|list| {
              for cat in FeedbackCategory::ALL {
                list
                  .spawn((
                    Node {
                      border: UiRect::all(Val::Px(1.0)),
                      padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
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

          // Tekstinput-område
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
                  button(styles.surface, styles.border),
                ))
                .with_children(|field| {
                  // Placeholder
                  field
                    .spawn((
                      Text::default(),
                      Node {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                      },
                      PickingBehavior {
                        should_block_lower: false,
                        ..default()
                      },
                      PlaceholderTextRoot,
                    ))
                    .with_children(|t| {
                      t.spawn((
                        TextSpan::new("Describe your feedback.. (what happened, where, expected vs actual)"),
                        TextFont::from_font_size(14.),
                        TextColor(styles.text_secondary.with_alpha(0.4)),
                      ));
                    });
                  // Faktisk innhold
                  field
                    .spawn((
                      Node {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                      },
                      Text::default(),
                      PickingBehavior {
                        should_block_lower: false,
                        ..default()
                      },
                    ))
                    .with_children(|t| {
                      t.spawn((
                        TextSpan::default(),
                        MessageTextRoot,
                        TextColor(styles.text_secondary),
                        TextFont::from_font_size(14.),
                      ));
                    });
                });
            });

          // Toggle + submit
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

          // Buttons
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

/* ========= Systems ========= */

// F2 for å åpne/lukke panelet
fn toggle_panel_visibility_with_key(keys: Res<ButtonInput<KeyCode>>, mut props: ResMut<FeedbackPanelProps>) {
  if keys.just_pressed(KeyCode::F2) {
    props.visible = !props.visible;
  }
}

// Synk display med visible
fn panel_visibility_sync(props: Res<FeedbackPanelProps>, mut q: Query<&mut Node, With<FeedbackPanel>>) {
  if !props.is_changed() {
    return;
  }
  if let Ok(mut node) = q.get_single_mut() {
    node.display = if props.visible { Display::Flex } else { Display::None };
  }
}

// Handle hover and click states
fn handle_hover_and_click_styles(
  mut commands: Commands,
  mut q: Query<
    (
      &Interaction,
      Entity,
      &mut BackgroundColor,
      &mut BorderColor,
      Option<&ButtonHoverStyle>,
      Option<&ButtonPressedStyle>,
      Option<&OriginalButtonStyles>,
      Has<HoldPressed>,
      Has<Active>,
    ),
    (Changed<Interaction>, Or<(With<ButtonHoverStyle>, With<ButtonPressedStyle>)>),
  >,
) {
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

// Klikk på input = fokus
fn input_focus_via_interaction(
  styles: Res<FeedbackPanelStyles>,
  mut q: Query<(&Interaction, &mut InputState, &mut BorderColor), (Changed<Interaction>, With<MessageInput>)>,
) {
  for (interaction, mut state, mut bd_color) in &mut q {
    if *interaction == Interaction::Pressed {
      bd_color.0 = styles.accent;
      state.focused = true;
    }
  }
}

// Klikk på andre knapper = defokus
fn input_defocus_when_other_buttons_pressed(
  mut commands: Commands,
  styles: Res<FeedbackPanelStyles>,
  mut q_input: Query<(Entity, &mut InputState), With<MessageInput>>,
  q_buttons: Query<&Interaction, (With<Button>, Changed<Interaction>, Without<MessageInput>)>,
) {
  let Ok((entity, mut input)) = q_input.get_single_mut() else {
    return;
  };
  for interaction in &q_buttons {
    if *interaction == Interaction::Pressed {
      commands
        .entity(entity)
        .insert(button(styles.surface, styles.border))
        .remove::<Active>();
      input.focused = false;
    }
  }
}

// Skriving (KeyboardInput): Enter = ny linje, Backspace, Space, og simple char mapping
fn input_type_chars(mut q_input: Query<&mut InputState, With<MessageInput>>, mut key_evr: EventReader<KeyboardInput>) {
  let Ok(mut state) = q_input.get_single_mut() else {
    return;
  };
  if !state.focused {
    return;
  }

  for ev in key_evr.read() {
    if !ev.state.is_pressed() {
      continue;
    }

    match ev.key_code {
      KeyCode::Enter => state.content.push('\n'),
      KeyCode::Backspace => {
        state.content.pop();
      },
      KeyCode::Space => state.content.push(' '),
      code => {
        if let Some(ch) = keycode_to_char(code, &ev.logical_key) {
          state.content.push(ch);
        }
      },
      _ => {},
    }
  }
}

// Veldig enkel mapping. For full støtte: bruk ev.logical (Key::Character) der den finnes.
fn keycode_to_char(code: KeyCode, logical: &Key) -> Option<char> {
  if let Key::Character(s) = logical {
    return s.chars().next();
  }
  use KeyCode::*;
  match code {
    KeyA => Some('a'),
    KeyB => Some('b'),
    KeyC => Some('c'),
    KeyD => Some('d'),
    KeyE => Some('e'),
    KeyF => Some('f'),
    KeyG => Some('g'),
    KeyH => Some('h'),
    KeyI => Some('i'),
    KeyJ => Some('j'),
    KeyK => Some('k'),
    KeyL => Some('l'),
    KeyM => Some('m'),
    KeyN => Some('n'),
    KeyO => Some('o'),
    KeyP => Some('p'),
    KeyQ => Some('q'),
    KeyR => Some('r'),
    KeyS => Some('s'),
    KeyT => Some('t'),
    KeyU => Some('u'),
    KeyV => Some('v'),
    KeyW => Some('w'),
    KeyX => Some('x'),
    KeyY => Some('y'),
    KeyZ => Some('z'),
    Digit0 => Some('0'),
    Digit1 => Some('1'),
    Digit2 => Some('2'),
    Digit3 => Some('3'),
    Digit4 => Some('4'),
    Digit5 => Some('5'),
    Digit6 => Some('6'),
    Digit7 => Some('7'),
    Digit8 => Some('8'),
    Digit9 => Some('9'),
    Minus => Some('-'),
    Equal => Some('='),
    BracketLeft => Some('['),
    BracketRight => Some(']'),
    Semicolon => Some(';'),
    Comma => Some(','),
    Period => Some('.'),
    Slash => Some('/'),
    _ => None,
  }
}

// Oppdater visning av tekst + placeholder via TextUiWriter
fn refresh_input_text(
  q_state: Query<&InputState, (With<MessageInput>, Changed<InputState>)>,
  mut msg_root_q: Query<&mut TextSpan, With<MessageTextRoot>>,
  mut ph_vis_q: Query<&mut Visibility, With<PlaceholderTextRoot>>,
) {
  let Ok(state) = q_state.get_single() else {
    return;
  };

  if let Ok(mut root) = msg_root_q.get_single_mut() {
    **root = state.content.clone();
  }

  if let Ok(mut vis) = ph_vis_q.get_single_mut() {
    *vis = if state.content.is_empty() {
      Visibility::Visible
    } else {
      Visibility::Hidden
    };
  }
}

// Rating
fn rating_click_system(
  mut form: ResMut<FeedbackFormState>,
  q: Query<(&Interaction, &RatingStar), (With<Button>, Changed<Interaction>)>,
) {
  for (interaction, RatingStar(v)) in &q {
    if *interaction == Interaction::Pressed {
      form.rating = *v;
    }
  }
}

// Åpne/lukke kategori-dropdown
fn category_toggle_system(
  mut form: ResMut<FeedbackFormState>,
  q: Query<&Interaction, (With<CategoryButton>, Changed<Interaction>)>,
) {
  for interaction in &q {
    if *interaction == Interaction::Pressed {
      form.dropdown_open = !form.dropdown_open;
    }
  }
}

// Plukke kategori + oppdatere knapptittel
fn category_pick_system(
  mut form: ResMut<FeedbackFormState>,
  q_items: Query<(&Interaction, &CategoryItem), (With<Button>, Changed<Interaction>)>,
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
fn dropdown_visibility_sync(form: Res<FeedbackFormState>, mut q: Query<&mut Node, With<CategoryList>>) {
  if !form.is_changed() {
    return;
  }
  if let Ok(mut n) = q.get_single_mut() {
    n.display = if form.dropdown_open {
      Display::Flex
    } else {
      Display::None
    };
  }
}

// Screenshot toggle
fn screenshot_toggle_click_system(
  styles: Res<FeedbackPanelStyles>,
  mut form: ResMut<FeedbackFormState>,
  mut q: Query<(&Interaction, &mut BackgroundColor), (With<ScreenshotToggle>, Changed<Interaction>)>,
  mut q_text_root: Query<(&mut TextSpan, &mut TextColor), With<ScreenshotToggleText>>,
) {
  q.iter_mut().for_each(|(interaction, mut bg_color)| {
    if *interaction == Interaction::Pressed {
      form.include_screenshot = !form.include_screenshot;
      bg_color.0 = if form.include_screenshot {
        styles.accent
      } else {
        styles.surface
      };
      if let Ok((mut root, mut color)) = q_text_root.get_single_mut() {
        **root = format!("Include screenshot: {}", if form.include_screenshot { "Yes" } else { "No" });
        color.0 = if form.include_screenshot {
          styles.text_primary
        } else {
          styles.text_secondary
        };
      }
    }
  });
}

// Submit
fn submit_click_system(
  q: Query<&Interaction, (With<SubmitButton>, Changed<Interaction>)>,
  q_input: Query<&InputState, With<MessageInput>>,
  form: Res<FeedbackFormState>,
  mut props: ResMut<FeedbackPanelProps>,
) {
  for interaction in &q {
    if *interaction != Interaction::Pressed {
      continue;
    }

    let msg = q_input
      .get_single()
      .map(|s| s.content.trim().to_string())
      .unwrap_or_default();
    if msg.is_empty() {
      // TODO: vis toast
      return;
    }

    let screenshot_url: Option<String> = if form.include_screenshot {
      // TODO: koble til faktisk screenshot-pipeline (returner URL eller legg ved opplasting)
      None
    } else {
      None
    };

    let payload = json!({
        "message": msg,
        "category": form.category.label(),
        "rating": form.rating,
        "screenshot_url": screenshot_url,
    });

    dbg!(&payload);
    // let ok = indigauge_client::send_feedback(payload);
    // if !ok {
    //   // TODO: toast "failed to enqueue"
    // } else {
    //   // valgfritt: lukk panelet eller tøm innhold
    //   visible.0 = false;
    // }

    props.visible = false;
  }
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
  mut mouse_wheel_events: EventReader<MouseWheel>,
  hover_map: Res<HoverMap>,
  mut scrolled_node_query: Query<&mut ScrollPosition>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
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
