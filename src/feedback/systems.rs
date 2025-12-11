use bevy::{
  input::mouse::{MouseScrollUnit, MouseWheel},
  picking::focus::HoverMap,
  prelude::*,
};
use bevy_text_edit::TextEditable;

use crate::{
  feedback::observers::{
    observe_cancel_click, observe_category_dropdown_click, observe_category_item_click,
    observe_screenshot_toggle_click, observe_submit_click,
  },
  feedback::{components::*, helpers::*, resources::*, types::FeedbackCategory},
  utils::select,
};

const LINE_HEIGHT: f32 = 21.;

pub fn despawn_feedback_panel(mut commands: Commands, query: Query<Entity, With<FeedbackPanel>>) {
  for entity in &query {
    commands.entity(entity).despawn_recursive();
  }
}

pub fn toggle_panel_visibility_with_key(
  mut commands: Commands,
  keys: Res<ButtonInput<KeyCode>>,
  toggle_button: Res<FeedbackKeyCodeToggle>,
) {
  if keys.just_pressed(toggle_button.0) {
    commands.insert_resource(FeedbackPanelProps::default());
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
          if let Some(mut ecm) = commands.get_entity(entity) {
            ecm.try_insert_if_new(OriginalButtonStyles {
              background: bg_color.0,
              border: border_color.0,
            });
          }

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

          if hold_after_press && let Some(mut ecm) = commands.get_entity(entity) {
            ecm.try_insert(Active);
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

// Synk dropdown synlighet
pub fn dropdown_visibility_sync(form: Res<FeedbackFormState>, mut q: Query<&mut Node, With<CategoryList>>) {
  if let Ok(mut n) = q.get_single_mut() {
    n.display = select(Display::Flex, Display::None, form.dropdown_open);
  }
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
        align_items: props.spawn_position.align_items(),
        justify_content: props.spawn_position.justify_content(),
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
            margin: props.position_margin,
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
          if let Some(title) = &props.title {
            child_panel
              .spawn((Text::default(), Node::default()))
              .with_children(|t| {
                t.spawn((TextSpan::new(title), TextFont::from_font_size(22.), TextColor(styles.text_primary)));
              });
          }

          if let Some(question) = &props.question {
            let size = select(18., 22., props.title.is_some());
            let color = select(styles.text_secondary, styles.text_primary, props.title.is_some());

            child_panel
              .spawn((Text::default(), Node::default()))
              .with_children(|t| {
                t.spawn((Text::new(question), QuestionTextRoot, TextFont::from_font_size(size), TextColor(color)));
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
                  })
                  .observe(observe_category_dropdown_click);
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
                    })
                    .observe(observe_category_item_click);
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
                      max_length: 1000,
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
                    b.spawn((
                      Text::new("Include screenshot: "),
                      TextFont::from_font_size(14.),
                      TextColor(styles.text_secondary),
                    ));
                    b.spawn((Text::default(), Node::default())).with_children(|t| {
                      t.spawn((
                        TextSpan::new("No"),
                        ScreenshotToggleText,
                        TextFont::from_font_size(14.),
                        TextColor(styles.secondary),
                      ));
                    });
                  })
                  .observe(observe_screenshot_toggle_click);
              });
          }

          // Submit and cancel buttons
          child_panel
            .spawn((
              Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceAround,
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
                })
                .observe(observe_cancel_click);

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
                    t.spawn((
                      TextSpan::new("Submit Feedback"),
                      TextFont::from_font_size(16.),
                      TextColor(styles.text_primary),
                    ));
                  });
                })
                .observe(observe_submit_click);
            });
        });
    });
}
