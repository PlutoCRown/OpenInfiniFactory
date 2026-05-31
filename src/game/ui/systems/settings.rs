pub fn update_settings_text_ui(
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    i18n: Res<I18n>,
    mut settings_texts: Query<(&SettingsText, Option<&ChildOf>, &mut Text)>,
    key_buttons: Query<&KeyBindingButton>,
) {
    for (settings_text, parent, mut text) in &mut settings_texts {
        text.0 = match settings_text.0 {
            SettingsTextKind::KeyBinding => {
                let Some(parent) = parent else {
                    continue;
                };
                let Ok(button) = key_buttons.get(parent.parent()) else {
                    continue;
                };
                let suffix = pending_key_bind
                    .0
                    .filter(|pending| *pending == button.0)
                    .map(|_| "...")
                    .unwrap_or(config.input(button.0).name());
                format!("{}: {suffix}", i18n.text(button.0.label_key()))
            }
        };
    }
}

pub fn update_settings_sliders_ui(
    settings: Res<GameSettings>,
    active_slider: Res<ActiveSettingsSlider>,
    mut slider_fills: Query<
        (&SettingsSliderFill, &mut Node),
        (Without<SettingsSliderKnob>, Without<SettingsDropdownList>),
    >,
    mut slider_knobs: Query<
        (&SettingsSliderKnob, &mut Node),
        (Without<SettingsSliderFill>, Without<SettingsDropdownList>),
    >,
    slider_values: Query<
        (Entity, &SettingsAction, &SliderValue, &CoreSliderDragState),
        With<Slider>,
    >,
    mut commands: Commands,
) {
    for (entity, action, value, drag_state) in &slider_values {
        if drag_state.dragging {
            continue;
        }
        if let SettingsAction::Field(field) = *action {
            if active_slider.0 == Some(field) {
                continue;
            }
            let next_value = field.percent(&settings);
            if (value.0 - next_value).abs() > 0.01 {
                commands.entity(entity).insert(SliderValue(next_value));
            }
        }
    }

    for (fill, mut style) in &mut slider_fills {
        let percent = live_slider_percent(fill.0, &settings, &active_slider, &slider_values);
        style.width = Val::Percent(percent);
    }

    for (knob, mut style) in &mut slider_knobs {
        let percent = live_slider_percent(knob.0, &settings, &active_slider, &slider_values);
        style.left = Val::Percent(percent);
    }
}

pub fn update_settings_slider_drag_ui(
    slider_values: Query<
        (&SettingsAction, &SliderValue, &SliderRange),
        (With<Slider>, Changed<SliderValue>),
    >,
    mut slider_fills: Query<
        (&SettingsSliderFill, &mut Node),
        (Without<SettingsSliderKnob>, Without<SettingsDropdownList>),
    >,
    mut slider_knobs: Query<
        (&SettingsSliderKnob, &mut Node),
        (Without<SettingsSliderFill>, Without<SettingsDropdownList>),
    >,
) {
    for (action, value, range) in &slider_values {
        let SettingsAction::Field(field) = *action else {
            continue;
        };
        let percent = (range.thumb_position(value.0) * 100.0).clamp(0.0, 100.0);

        for (fill, mut style) in &mut slider_fills {
            if fill.0 == field {
                style.width = Val::Percent(percent);
            }
        }

        for (knob, mut style) in &mut slider_knobs {
            if knob.0 == field {
                style.left = Val::Percent(percent);
            }
        }
    }
}

pub fn update_settings_dropdowns_ui(
    config: Res<GameConfig>,
    settings: Res<GameSettings>,
    open_dropdown: Res<OpenSettingsDropdown>,
    i18n: Res<I18n>,
    mut texts: ParamSet<(
        Query<
            (&SettingsDropdownLabel, &mut Text),
            (
                Without<SettingsText>,
                Without<SettingsValueText>,
            ),
        >,
        Query<
            (&SettingsValueText, &mut Text),
            (
                Without<SettingsText>,
                Without<SettingsDropdownLabel>,
            ),
        >,
    )>,
    mut dropdown_lists: Query<
        (&SettingsDropdownList, &mut Node),
        (Without<SettingsSliderFill>, Without<SettingsSliderKnob>),
    >,
    mut dropdown_z: Query<(
        &mut ZIndex,
        Option<&SettingsDropdownList>,
        Option<&SettingsDropdownRoot>,
        Option<&SettingsDropdownRow>,
    )>,
) {
    for (label, mut text) in &mut texts.p0() {
        text.0 = match label.0 {
            super::types::SettingsDropdown::Language => i18n.language().native_name().to_string(),
            super::types::SettingsDropdown::PlaceSelectionMode => {
                i18n.text(config.place_selection_mode.label_key())
            }
            super::types::SettingsDropdown::DeleteSelectionMode => {
                i18n.text(config.delete_selection_mode.label_key())
            }
        };
    }

    for (value, mut text) in &mut texts.p1() {
        text.0 = value.0.display(&settings, &i18n);
    }

    for (list, mut style) in &mut dropdown_lists {
        let open = open_dropdown.0 == Some(list.0);
        style.display = if open { Display::Flex } else { Display::None };
    }

    for (mut z_index, list, root, row) in &mut dropdown_z {
        if let Some(list) = list {
            *z_index = if open_dropdown.0 == Some(list.0) {
                ZIndex(900)
            } else {
                ZIndex(500)
            };
        } else if let Some(root) = root {
            *z_index = if open_dropdown.0 == Some(root.0) {
                ZIndex(850)
            } else {
                ZIndex(300)
            };
        } else if let Some(row) = row {
            *z_index = if open_dropdown.0 == Some(row.0) {
                ZIndex(800)
            } else {
                ZIndex(300)
            };
        }
    }
}

pub fn update_settings_tabs_ui(
    settings_tab: Res<SettingsTab>,
    hover: Res<UiHoverState>,
    mut tab_buttons: Query<
        (
            Entity,
            &SettingsAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    for (entity, action, mut background, mut border) in &mut tab_buttons {
        let selected = matches!(
            (*action, *settings_tab),
            (SettingsAction::TabGameplay, SettingsTab::Gameplay)
                | (SettingsAction::TabKeyBindings, SettingsTab::KeyBindings)
        );
        let hovered = hover.entity == Some(entity);
        if selected {
            *background = Color::srgb(0.56, 0.56, 0.56).into();
            *border = pressed_border();
        } else if matches!(
            *action,
            SettingsAction::TabGameplay | SettingsAction::TabKeyBindings
        ) {
            if hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        } else {
            if hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        }
    }
}

fn live_slider_percent(
    field: SettingsField,
    settings: &GameSettings,
    active_slider: &ActiveSettingsSlider,
    slider_values: &Query<
        (Entity, &SettingsAction, &SliderValue, &CoreSliderDragState),
        With<Slider>,
    >,
) -> f32 {
    slider_values
        .iter()
        .find_map(|(_, action, value, drag_state)| {
            ((drag_state.dragging || active_slider.0 == Some(field))
                && *action == SettingsAction::Field(field))
            .then_some(value.0.clamp(0.0, 100.0))
        })
        .unwrap_or_else(|| field.percent(settings))
}
