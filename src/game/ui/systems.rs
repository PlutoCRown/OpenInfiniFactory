use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, TeleportRenameState,
};
use crate::game::world::grid::ConverterMode;
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::i18n::I18n;
use crate::shared::save::{SaveKind, SaveState};

use super::components::{
    BUTTON_BG, BUTTON_BORDER, BUTTON_HOVER_BG, BUTTON_HOVER_BORDER, BUTTON_PRESSED_BG,
};
use super::types::{
    BackpackPanel, CarriedIcon, CarriedItem, CarriedLabel, ConverterInputRow, ConverterInputText,
    ConverterModeText, ConverterOutputText, ConverterPanel, Crosshair, CurrentSaveText,
    DeleteSelectionModeText, FovText, GeneratorMaterialText, GeneratorPanel, GeneratorPeriodText,
    HotbarText, InGameHudStyle, InGameHudVisibility, InventoryItems, InventorySlot, InventoryTitle,
    KeyBindingButton, KeyBindingLabel, LabelerColorText, LabelerPanel, LocalizedText,
    MainMenuPanel, OpenSettingsDropdown, PauseAction, PausePanel, PendingKeyBind,
    PlaceSelectionModeText, SaveListAction, SaveListLabel, SaveListPanel, SaveListTitle,
    ScrollContainer, ScrollContent, SettingsAction, SettingsDropdownLabel, SettingsDropdownList,
    SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsPanel, SettingsSlider,
    SettingsSliderFill, SettingsSliderKnob, SettingsStatusText, SettingsTab, SettingsValue,
    SettingsValueText, SimulationText, SlotArea, SlotLabel, TeleportNameText, TeleportPairText,
    TeleportPanel, UiScaleText,
};
use super::widgets::{short_item_name, slot_color};

#[derive(Resource, Clone)]
pub struct UiFont(pub Handle<Font>);

pub fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFont(asset_server.load("fonts/PingFangSC-Regular.ttf")));
}

pub fn apply_ui_font(ui_font: Option<Res<UiFont>>, mut text_query: Query<&mut Text, Added<Text>>) {
    let Some(ui_font) = ui_font else {
        return;
    };

    for mut text in &mut text_query {
        for section in &mut text.sections {
            section.style.font = ui_font.0.clone();
        }
    }
}

pub fn inventory_slot_clicks(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>),
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    config: Res<GameConfig>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mode: Res<GameMode>,
) {
    if *mode != GameMode::Inventory {
        return;
    }

    for (interaction, slot) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let pick_button = config
            .input(ConfigAction::Pick)
            .mouse_button()
            .unwrap_or(MouseButton::Middle);
        if mouse_buttons.just_pressed(pick_button) {
            if slot.area == SlotArea::Hotbar {
                inventory.hotbar[slot.index] = None;
                if placement.selected == slot.index {
                    carried.clear();
                }
            }
            placement.selection.clear();
            placement.edit_gesture = None;
            continue;
        }

        let clicked_item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };

        if slot.area == SlotArea::Hotbar {
            if let Some(item) = carried.item() {
                inventory.hotbar[slot.index] = Some(item);
                placement.selected = slot.index;
                carried.clear();
            } else {
                if let Some(item) = clicked_item {
                    if place_in_backpack(&mut inventory, item) {
                        inventory.hotbar[slot.index] = None;
                        carried.clear();
                    } else {
                        carried.set(Some(item));
                    }
                } else {
                    carried.clear();
                }
                placement.selected = slot.index;
            }
        } else {
            if let Some(item) = carried.take() {
                if inventory.backpack[slot.index].is_none() {
                    inventory.backpack[slot.index] = Some(item);
                } else {
                    let previous = inventory.backpack[slot.index].replace(item);
                    carried.set(previous);
                }
            } else {
                carried.set(clicked_item);
            }
        }
        placement.selection.clear();
        placement.edit_gesture = None;
    }
}

fn place_in_backpack(inventory: &mut InventoryItems, item: super::types::InventoryItem) -> bool {
    if inventory.backpack.iter().any(|slot| *slot == Some(item)) {
        return true;
    }
    if let Some(slot) = inventory.backpack.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(item);
        return true;
    }
    false
}

pub fn update_button_hover_ui(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<Button>,
            Without<InventorySlot>,
            Without<SettingsAction>,
            Without<SaveListAction>,
        ),
    >,
) {
    for (interaction, mut background, mut border) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                *background = BUTTON_PRESSED_BG.into();
                *border = BUTTON_HOVER_BORDER.into();
            }
            Interaction::Hovered => {
                *background = BUTTON_HOVER_BG.into();
                *border = BUTTON_HOVER_BORDER.into();
            }
            Interaction::None => {
                *background = BUTTON_BG.into();
                *border = BUTTON_BORDER.into();
            }
        }
    }
}

fn pause_action_visible(mode: GameMode, save_state: &SaveState, action: PauseAction) -> bool {
    let confirming = mode == GameMode::ConfirmSaveSolutionBeforeEdit;
    if confirming {
        return matches!(
            action,
            PauseAction::ConfirmSaveSolutionAndEdit
                | PauseAction::DiscardSolutionAndEdit
                | PauseAction::CancelEditSwitch
        );
    }

    match action {
        PauseAction::ConfirmSaveSolutionAndEdit
        | PauseAction::DiscardSolutionAndEdit
        | PauseAction::CancelEditSwitch => false,
        PauseAction::ResetSolution => save_state.current_kind == Some(SaveKind::Solution),
        _ => true,
    }
}

pub fn update_status_ui(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    settings: Res<GameSettings>,
    save_state: Res<SaveState>,
    i18n: Res<I18n>,
    mut hotbar: Query<&mut Text, (With<HotbarText>, Without<SlotLabel>, Without<CarriedLabel>)>,
    mut inventory_title: Query<
        &mut Text,
        (
            With<InventoryTitle>,
            Without<HotbarText>,
            Without<SlotLabel>,
            Without<CarriedLabel>,
            Without<FovText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut fov_text: Query<
        &mut Text,
        (
            With<FovText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<InventoryTitle>,
            Without<UiScaleText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut ui_scale_text: Query<
        &mut Text,
        (
            With<UiScaleText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut simulation_text: Query<
        &mut Text,
        (
            With<SimulationText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<UiScaleText>,
            Without<CurrentSaveText>,
        ),
    >,
    mut current_save_text: Query<
        &mut Text,
        (
            With<CurrentSaveText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<SimulationText>,
        ),
    >,
) {
    if let Ok(mut text) = hotbar.get_single_mut() {
        let selected = inventory.hotbar[placement.selected]
            .map(|kind| i18n.text(kind.name_key()))
            .unwrap_or_else(|| i18n.text("empty"));
        text.sections[0].value = i18n.fmt(
            "status.hotbar",
            &[
                ("mode", builder_mode_name(*builder_mode, &i18n)),
                ("selected", selected),
                ("facing", i18n.text(placement.facing.name_key())),
            ],
        );
    }

    if let Ok(mut text) = inventory_title.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "inventory.title",
            &[("mode", builder_mode_name(*builder_mode, &i18n))],
        );
    }

    if let Ok(mut text) = fov_text.get_single_mut() {
        text.sections[0].value = format!("FOV {:.0}", settings.fov_degrees);
    }

    if let Ok(mut text) = ui_scale_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "settings.ui_scale",
            &[("scale", format!("{:.1}", settings.ui_scale))],
        );
    }

    if let Ok(mut text) = simulation_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "status.simulation",
            &[
                ("mode", builder_mode_name(*builder_mode, &i18n)),
                ("turns", simulation.turn.to_string()),
                (
                    "state",
                    i18n.text(if simulation.running {
                        "state.playing"
                    } else {
                        "state.paused"
                    }),
                ),
                ("speed", format!("{:.0}", simulation.speed)),
            ],
        );
    }

    if let Ok(mut text) = current_save_text.get_single_mut() {
        text.sections[0].value = save_state
            .current
            .as_ref()
            .map(|name| i18n.fmt("save.world", &[("name", name.clone())]))
            .unwrap_or_else(|| i18n.text("save.no_world_loaded"));
    }
}

pub fn update_carried_item_ui(
    carried: Res<CarriedItem>,
    i18n: Res<I18n>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut icon: Query<(&mut Style, &mut BackgroundColor), With<CarriedIcon>>,
    mut label: Query<&mut Text, With<CarriedLabel>>,
) {
    let Ok((mut style, mut background)) = icon.get_single_mut() else {
        return;
    };

    let Some(item) = carried.item() else {
        style.display = Display::None;
        if let Ok(mut text) = label.get_single_mut() {
            text.sections[0].value.clear();
        }
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        style.display = Display::None;
        return;
    };

    style.display = Display::Flex;
    style.left = Val::Px(cursor.x + 4.0);
    style.top = Val::Px(cursor.y + 4.0);
    *background = slot_color(item).with_alpha(0.9).into();

    if let Ok(mut text) = label.get_single_mut() {
        text.sections[0].value = i18n.text(short_item_name(item));
    }
}

pub fn update_scroll_containers(
    mode: Res<GameMode>,
    settings_tab: Res<SettingsTab>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut containers: Query<(&mut ScrollContainer, &Children, &Node)>,
    mut contents: Query<(&mut Style, &Node), With<ScrollContent>>,
) {
    if *mode != GameMode::Settings || *settings_tab != SettingsTab::KeyBindings {
        return;
    }

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();

    for (mut container, children, node) in &mut containers {
        let Some(child) = children
            .iter()
            .find(|child| contents.get(**child).is_ok())
            .copied()
        else {
            continue;
        };
        let Ok((mut style, content_node)) = contents.get_mut(child) else {
            continue;
        };

        container.max_offset = (content_node.size().y - node.size().y).max(0.0);
        if wheel_delta.abs() > f32::EPSILON {
            container.offset =
                (container.offset - wheel_delta * 32.0).clamp(0.0, container.max_offset);
        } else {
            container.offset = container.offset.clamp(0.0, container.max_offset);
        }
        style.top = Val::Px(-container.offset);
    }
}

pub fn update_generator_ui(
    placement: Res<PlacementState>,
    world: Res<crate::game::world::grid::WorldBlocks>,
    i18n: Res<I18n>,
    mut generator_period_text: Query<
        &mut Text,
        (With<GeneratorPeriodText>, Without<GeneratorMaterialText>),
    >,
    mut generator_material_text: Query<
        &mut Text,
        (With<GeneratorMaterialText>, Without<GeneratorPeriodText>),
    >,
) {
    let Some(pos) = placement.generator_panel else {
        return;
    };

    let generator_settings = world.generator_settings(pos);
    if let Ok(mut text) = generator_period_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "generator.period",
            &[("period", generator_settings.period.to_string())],
        );
    }
    if let Ok(mut text) = generator_material_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "generator.material",
            &[(
                "material",
                i18n.text(generator_settings.material.name_key()),
            )],
        );
    }
}

pub fn update_labeler_ui(
    placement: Res<PlacementState>,
    world: Res<crate::game::world::grid::WorldBlocks>,
    i18n: Res<I18n>,
    mut labeler_color_text: Query<&mut Text, With<LabelerColorText>>,
) {
    let Some(pos) = placement.labeler_panel else {
        return;
    };

    let labeler_settings = world.labeler_settings(pos);
    if let Ok(mut text) = labeler_color_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "labeler.color",
            &[("color", i18n.text(labeler_settings.color.name_key()))],
        );
    }
}

pub fn update_converter_ui(
    placement: Res<PlacementState>,
    world: Res<crate::game::world::grid::WorldBlocks>,
    i18n: Res<I18n>,
    mut converter_mode_text: Query<
        &mut Text,
        (
            With<ConverterModeText>,
            Without<ConverterInputText>,
            Without<ConverterOutputText>,
        ),
    >,
    mut converter_input_text: Query<
        &mut Text,
        (
            With<ConverterInputText>,
            Without<ConverterModeText>,
            Without<ConverterOutputText>,
        ),
    >,
    mut converter_output_text: Query<
        &mut Text,
        (
            With<ConverterOutputText>,
            Without<ConverterModeText>,
            Without<ConverterInputText>,
        ),
    >,
    mut converter_input_row: Query<&mut Style, With<ConverterInputRow>>,
) {
    let Some(pos) = placement.converter_panel else {
        return;
    };

    let settings = world.converter_settings(pos);
    if let Ok(mut text) = converter_mode_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "converter.mode",
            &[("mode", i18n.text(settings.mode.name_key()))],
        );
    }
    if let Ok(mut text) = converter_input_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "converter.input",
            &[("material", i18n.text(settings.input.name_key()))],
        );
    }
    if let Ok(mut text) = converter_output_text.get_single_mut() {
        text.sections[0].value = i18n.fmt(
            "converter.output",
            &[("material", i18n.text(settings.output.name_key()))],
        );
    }
    for mut style in &mut converter_input_row {
        style.display = if settings.mode == ConverterMode::SpecificInput {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_teleport_ui(
    placement: Res<PlacementState>,
    rename_state: Res<TeleportRenameState>,
    world: Res<crate::game::world::grid::WorldBlocks>,
    i18n: Res<I18n>,
    mut teleport_name_text: Query<&mut Text, (With<TeleportNameText>, Without<TeleportPairText>)>,
    mut teleport_pair_text: Query<&mut Text, (With<TeleportPairText>, Without<TeleportNameText>)>,
) {
    let Some(pos) = placement.teleport_panel else {
        return;
    };

    let settings = world.teleport_settings(pos);
    if let Ok(mut text) = teleport_name_text.get_single_mut() {
        let name = if rename_state.editing == Some(pos) {
            i18n.fmt("teleport.editing", &[("name", rename_state.buffer.clone())])
        } else {
            i18n.fmt("teleport.name", &[("name", settings.name.clone())])
        };
        text.sections[0].value = name;
    }
    if let Ok(mut text) = teleport_pair_text.get_single_mut() {
        let pair = settings
            .pair
            .map(|pair| world.teleport_settings(pair).name)
            .unwrap_or_else(|| i18n.text("teleport.none"));
        text.sections[0].value = i18n.fmt("teleport.pair", &[("pair", pair)]);
    }
}

fn builder_mode_name(mode: BuilderMode, i18n: &I18n) -> String {
    match mode {
        BuilderMode::Edit => i18n.text("mode.edit"),
        BuilderMode::Play => i18n.text("mode.play"),
    }
}

pub fn update_settings_text_ui(
    config: Res<GameConfig>,
    settings_tab: Res<SettingsTab>,
    pending_key_bind: Res<PendingKeyBind>,
    i18n: Res<I18n>,
    mut settings_status: Query<
        &mut Text,
        (
            With<SettingsStatusText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<FovText>,
            Without<PlaceSelectionModeText>,
            Without<DeleteSelectionModeText>,
            Without<SimulationText>,
            Without<CurrentSaveText>,
            Without<KeyBindingLabel>,
            Without<SettingsDropdownLabel>,
            Without<SettingsValueText>,
        ),
    >,
    mut key_labels: Query<
        (&Parent, &mut Text),
        (
            With<KeyBindingLabel>,
            Without<SettingsStatusText>,
            Without<SettingsDropdownLabel>,
            Without<SettingsValueText>,
        ),
    >,
    mut place_mode_text: Query<
        &mut Text,
        (
            With<PlaceSelectionModeText>,
            Without<SettingsStatusText>,
            Without<KeyBindingLabel>,
            Without<DeleteSelectionModeText>,
            Without<SettingsDropdownLabel>,
            Without<SettingsValueText>,
        ),
    >,
    mut delete_mode_text: Query<
        &mut Text,
        (
            With<DeleteSelectionModeText>,
            Without<SettingsStatusText>,
            Without<KeyBindingLabel>,
            Without<PlaceSelectionModeText>,
            Without<SettingsDropdownLabel>,
            Without<SettingsValueText>,
        ),
    >,
    key_buttons: Query<&KeyBindingButton>,
) {
    if let Ok(mut text) = settings_status.get_single_mut() {
        let tab_name = match *settings_tab {
            SettingsTab::Gameplay => i18n.text("tab.gameplay"),
            SettingsTab::KeyBindings => i18n.text("tab.key_bindings"),
        };
        let pending = pending_key_bind
            .0
            .map(|action| {
                i18n.fmt(
                    "settings.pending_key",
                    &[("action", i18n.text(action.label_key()))],
                )
            })
            .unwrap_or_else(|| i18n.text("settings.rebind_hint"));
        text.sections[0].value = i18n.fmt(
            "settings.status",
            &[
                ("tab", tab_name),
                ("pending", pending),
                (
                    "simulate",
                    config.input(ConfigAction::Simulate).name().to_string(),
                ),
                (
                    "rollback",
                    config
                        .input(ConfigAction::RotateOrRollback)
                        .name()
                        .to_string(),
                ),
                (
                    "inventory",
                    config.input(ConfigAction::Inventory).name().to_string(),
                ),
                (
                    "alternate",
                    config.input(ConfigAction::Alternate).name().to_string(),
                ),
                (
                    "pause",
                    config.input(ConfigAction::Pause).name().to_string(),
                ),
            ],
        );
    }

    for (parent, mut text) in &mut key_labels {
        let Ok(button) = key_buttons.get(parent.get()) else {
            continue;
        };
        let suffix = pending_key_bind
            .0
            .filter(|pending| *pending == button.0)
            .map(|_| "...")
            .unwrap_or(config.input(button.0).name());
        text.sections[0].value = format!("{}: {suffix}", i18n.text(button.0.label_key()));
    }

    if let Ok(mut text) = place_mode_text.get_single_mut() {
        text.sections[0].value = i18n.text(config.place_selection_mode.label_key());
    }

    if let Ok(mut text) = delete_mode_text.get_single_mut() {
        text.sections[0].value = i18n.text(config.delete_selection_mode.label_key());
    }
}

pub fn update_settings_sliders_ui(
    settings: Res<GameSettings>,
    mut slider_fills: Query<
        (&SettingsSliderFill, &mut Style),
        (Without<SettingsSliderKnob>, Without<SettingsDropdownList>),
    >,
    mut slider_knobs: Query<
        (&SettingsSliderKnob, &mut Style),
        (Without<SettingsSliderFill>, Without<SettingsDropdownList>),
    >,
) {
    for (fill, mut style) in &mut slider_fills {
        style.width = Val::Percent(settings_slider_percent(fill.0, &settings));
    }

    for (knob, mut style) in &mut slider_knobs {
        style.left = Val::Percent(settings_slider_percent(knob.0, &settings));
    }
}

pub fn update_settings_dropdowns_ui(
    config: Res<GameConfig>,
    settings: Res<GameSettings>,
    open_dropdown: Res<OpenSettingsDropdown>,
    i18n: Res<I18n>,
    mut dropdown_labels: Query<
        (&SettingsDropdownLabel, &mut Text),
        (
            Without<SettingsStatusText>,
            Without<KeyBindingLabel>,
            Without<FovText>,
            Without<PlaceSelectionModeText>,
            Without<DeleteSelectionModeText>,
            Without<SettingsValueText>,
        ),
    >,
    mut value_texts: Query<
        (&SettingsValueText, &mut Text),
        (
            Without<SettingsStatusText>,
            Without<KeyBindingLabel>,
            Without<FovText>,
            Without<PlaceSelectionModeText>,
            Without<DeleteSelectionModeText>,
            Without<SettingsDropdownLabel>,
        ),
    >,
    mut dropdown_lists: Query<
        (&SettingsDropdownList, &mut Style),
        (Without<SettingsSliderFill>, Without<SettingsSliderKnob>),
    >,
) {
    for (label, mut text) in &mut dropdown_labels {
        text.sections[0].value = match label.0 {
            super::types::SettingsDropdown::Language => i18n.language().native_name().to_string(),
            super::types::SettingsDropdown::PlaceSelectionMode => {
                i18n.text(config.place_selection_mode.label_key())
            }
            super::types::SettingsDropdown::DeleteSelectionMode => {
                i18n.text(config.delete_selection_mode.label_key())
            }
        };
    }

    for (value, mut text) in &mut value_texts {
        text.sections[0].value = match value.0 {
            SettingsValue::Fov => format!("FOV {:.0}", settings.fov_degrees),
            SettingsValue::UiScale => i18n.fmt(
                "settings.ui_scale",
                &[("scale", format!("{:.1}", settings.ui_scale))],
            ),
            SettingsValue::Gravity => i18n.fmt(
                "settings.gravity_value",
                &[("scale", format!("{:.1}", settings.gravity_scale))],
            ),
        };
    }

    for (list, mut style) in &mut dropdown_lists {
        style.display = if open_dropdown.0 == Some(list.0) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_settings_tabs_ui(
    settings_tab: Res<SettingsTab>,
    mut tab_buttons: Query<
        (
            &SettingsAction,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    for (action, interaction, mut background, mut border) in &mut tab_buttons {
        let selected = matches!(
            (*action, *settings_tab),
            (SettingsAction::TabGameplay, SettingsTab::Gameplay)
                | (SettingsAction::TabKeyBindings, SettingsTab::KeyBindings)
        );
        if selected {
            *background = Color::srgba(0.24, 0.30, 0.34, 0.98).into();
            *border = Color::srgb(0.42, 0.72, 0.82).into();
        } else if matches!(
            *action,
            SettingsAction::TabGameplay | SettingsAction::TabKeyBindings
        ) {
            if *interaction == Interaction::Hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = BUTTON_HOVER_BORDER.into();
            } else {
                *background = Color::srgba(0.16, 0.17, 0.18, 0.96).into();
                *border = BUTTON_BORDER.into();
            }
        } else {
            match *interaction {
                Interaction::Pressed => {
                    *background = BUTTON_PRESSED_BG.into();
                    *border = BUTTON_HOVER_BORDER.into();
                }
                Interaction::Hovered => {
                    *background = BUTTON_HOVER_BG.into();
                    *border = BUTTON_HOVER_BORDER.into();
                }
                Interaction::None => {
                    *background = BUTTON_BG.into();
                    *border = BUTTON_BORDER.into();
                }
            }
        }
    }
}

fn settings_slider_percent(slider: SettingsSlider, settings: &GameSettings) -> f32 {
    match slider {
        SettingsSlider::Fov => ((settings.fov_degrees - 50.0) / 60.0 * 100.0).clamp(0.0, 100.0),
        SettingsSlider::UiScale => {
            ((settings.ui_scale - UI_SCALE_MIN) / (UI_SCALE_MAX - UI_SCALE_MIN) * 100.0)
                .clamp(0.0, 100.0)
        }
        SettingsSlider::Gravity => ((settings.gravity_scale - GRAVITY_SCALE_MIN)
            / (GRAVITY_SCALE_MAX - GRAVITY_SCALE_MIN)
            * 100.0)
            .clamp(0.0, 100.0),
    }
}

pub fn update_localized_ui(
    i18n: Res<I18n>,
    mut localized_text: Query<(&LocalizedText, &mut Text)>,
) {
    if !i18n.is_changed() {
        return;
    }

    for (localized, mut text) in &mut localized_text {
        text.sections[0].value = i18n.text(localized.key);
    }
}

pub fn update_panel_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    settings_tab: Res<SettingsTab>,
    mut style_sets: ParamSet<(
        Query<&mut Style, (With<MainMenuPanel>, Without<PauseAction>)>,
        Query<&mut Style, (With<SaveListPanel>, Without<PauseAction>)>,
        Query<&mut Style, (With<SettingsPanel>, Without<PauseAction>)>,
        Query<&mut Style, (With<SettingsGameplayGroup>, Without<PauseAction>)>,
        Query<&mut Style, (With<SettingsKeyBindingsGroup>, Without<PauseAction>)>,
        Query<&mut Style, (With<BackpackPanel>, Without<PauseAction>)>,
        Query<&mut Style, (With<PausePanel>, Without<PauseAction>)>,
        Query<&mut Style, (With<GeneratorPanel>, Without<PauseAction>)>,
    )>,
    mut pause_buttons: Query<(&PauseAction, &mut Style), With<Button>>,
) {
    for mut style in &mut style_sets.p0() {
        style.display = if *mode == GameMode::MainMenu {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p1() {
        style.display = if matches!(*mode, GameMode::SaveListMain | GameMode::SaveListPause) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p2() {
        style.display = if *mode == GameMode::Settings {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p3() {
        style.display = if *mode == GameMode::Settings && *settings_tab == SettingsTab::Gameplay {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p4() {
        style.display = if *mode == GameMode::Settings && *settings_tab == SettingsTab::KeyBindings
        {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p5() {
        style.display = if *mode == GameMode::Inventory {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p6() {
        style.display = if matches!(
            *mode,
            GameMode::Paused | GameMode::ConfirmSaveSolutionBeforeEdit
        ) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (action, mut style) in &mut pause_buttons {
        style.display = if pause_action_visible(*mode, &save_state, *action) {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p7() {
        style.display = if *mode == GameMode::GeneratorSettings {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_labeler_panel_visibility(
    mode: Res<GameMode>,
    mut labeler_panel: Query<&mut Style, With<LabelerPanel>>,
) {
    for mut style in &mut labeler_panel {
        style.display = if *mode == GameMode::LabelerSettings {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_converter_panel_visibility(
    mode: Res<GameMode>,
    mut converter_panel: Query<&mut Style, With<ConverterPanel>>,
) {
    for mut style in &mut converter_panel {
        style.display = if *mode == GameMode::ConverterSettings {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_teleport_panel_visibility(
    mode: Res<GameMode>,
    mut teleport_panel: Query<&mut Style, With<TeleportPanel>>,
) {
    for mut style in &mut teleport_panel {
        style.display = if *mode == GameMode::TeleportSettings {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_hud_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    mut hud_style: Query<&mut Style, With<InGameHudStyle>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility, With<Crosshair>>,
        Query<&mut Visibility, With<InGameHudVisibility>>,
    )>,
) {
    let has_world = save_state.current.is_some();

    for mut style in &mut hud_style {
        style.display = if has_world {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut visibility in &mut visibility_sets.p0() {
        *visibility = if has_world && *mode == GameMode::Playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in &mut visibility_sets.p1() {
        *visibility = if has_world {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn update_inventory_slots(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    i18n: Res<I18n>,
    mut slot_query: Query<
        (
            &InventorySlot,
            &Interaction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut Text, (With<SlotLabel>, Without<HotbarText>, Without<CarriedLabel>)>,
) {
    for (slot, interaction, children, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };

        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;
        let base_color = item
            .map(slot_color)
            .unwrap_or(Color::srgba(0.16, 0.16, 0.17, 0.92));
        *background = if *interaction == Interaction::Hovered && item.is_none() {
            Color::srgba(0.24, 0.26, 0.28, 0.96).into()
        } else if *interaction == Interaction::Hovered {
            base_color.with_alpha(1.0).into()
        } else {
            base_color.into()
        };
        *border = if selected_hotbar {
            Color::srgb(1.0, 1.0, 1.0).into()
        } else if *interaction == Interaction::Hovered {
            BUTTON_HOVER_BORDER.into()
        } else {
            Color::srgb(0.22, 0.22, 0.22).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(*child) {
                text.sections[0].value = item
                    .map(|kind| i18n.text(short_item_name(kind)))
                    .unwrap_or_default();
            }
        }
    }
}

pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    i18n: Res<I18n>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<SaveListTitle>>,
        Query<&mut Text, With<SaveListLabel>>,
    )>,
    mut slots: Query<
        (
            &SaveListAction,
            &Interaction,
            &Children,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
) {
    if let Ok(mut title) = text_sets.p0().get_single_mut() {
        title.sections[0].value = match *mode {
            GameMode::SaveListMain => i18n.text("save.title.main"),
            GameMode::SaveListPause => i18n.text("save.title.pause"),
            _ => i18n.text("save.title.default"),
        };
    }

    for (action, interaction, children, mut background) in &mut slots {
        let label = match *action {
            SaveListAction::Load(index) => save_state
                .slots
                .get(index)
                .map(|name| i18n.fmt("save.load", &[("name", name.clone())]))
                .unwrap_or_else(|| i18n.text("empty_slot")),
            SaveListAction::Back => i18n.text("button.back"),
        };

        let enabled_load = match *action {
            SaveListAction::Load(index) => save_state.slots.get(index).is_some(),
            SaveListAction::Back => true,
        };
        *background = if enabled_load && *interaction == Interaction::Hovered {
            BUTTON_HOVER_BG.into()
        } else if enabled_load {
            BUTTON_BG.into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = text_sets.p1().get_mut(*child) {
                text.sections[0].value = label.clone();
            }
        }
    }
}
