use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui_widgets::{CoreSliderDragState, Slider, SliderRange, SliderValue};
use bevy::window::PrimaryWindow;

use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, SolutionState,
    TeleportRenameState, WorldEntryMode,
};
use crate::game::world::blocks::BlockKind;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{ConfigAction, GameConfig};
use crate::shared::i18n::{I18n, Language};
use crate::shared::save::{SaveKind, SaveState};

use super::components::{
    hover_border, inset_border, menu_button, pressed_border, raised_border, BUTTON_BG,
    BUTTON_HOVER_BG, BUTTON_PRESSED_BG,
};
use super::types::{
    ActiveSettingsSlider, BackpackPanel, BlockPanelDropdown, BlockPanelDropdownLabel,
    BlockPanelDropdownList, CarriedIcon, CarriedItem, CarriedLabel, ConfirmDialogAction,
    ConfirmDialogKind, ConfirmDialogMessage, ConfirmDialogPanel, ConfirmDialogPrimaryLabel,
    ConfirmDialogSecondaryLabel, ConfirmDialogState, ConfirmDialogTitle, ConverterInputRow,
    Crosshair, CurrentSaveText, DeleteSelectionModeText, FovText, GeneratorPeriodText, HotbarText,
    InGameHudStyle, InGameHudVisibility, InventoryItems, InventorySlot, InventoryTitle,
    InventoryTooltip, InventoryTooltipText, KeyBindingButton, KeyBindingLabel, LocalizedText,
    MainMenuPanel, ModalScrim, OpenBlockPanelDropdown, OpenSettingsDropdown, PauseAction,
    PausePanel, PendingKeyBind, PlaceSelectionModeText, SaveListAction, SaveListLabel,
    SaveListPanel, SaveListTitle, ScrollContainer, ScrollContent, SettingsAction,
    SettingsDropdownLabel, SettingsDropdownList, SettingsDropdownRoot, SettingsDropdownRow,
    SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsSlider, SettingsSliderFill,
    SettingsSliderKnob, SettingsTab, SettingsValue, SettingsValueText, SimulationStatusText,
    SimulationText, SlotArea, SlotIcon, SlotLabel, TeleportAction, TeleportNameText,
    UiPanelBinding, UiPanelId, UiRuntime, UiScaleText,
};
use super::widgets::{short_item_name, slot_color};

#[derive(SystemParam)]
pub struct BlockPanelDropdownParams<'w, 's> {
    pub labels: Query<'w, 's, (&'static BlockPanelDropdownLabel, &'static mut Text)>,
    pub lists: Query<'w, 's, (&'static BlockPanelDropdownList, &'static mut Node)>,
    pub teleport_pair_list: Query<
        'w,
        's,
        (
            Entity,
            &'static BlockPanelDropdownList,
            Option<&'static Children>,
        ),
    >,
    pub teleport_pair_options: Query<'w, 's, Entity, With<TeleportAction>>,
}

#[derive(Resource, Clone)]
pub struct UiFont(pub Handle<Font>);

pub fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFont(asset_server.load("fonts/PingFangSC-Regular.ttf")));
}

pub fn apply_ui_font(
    ui_font: Option<Res<UiFont>>,
    mut text_query: Query<&mut TextFont, Added<Text>>,
) {
    let Some(ui_font) = ui_font else {
        return;
    };

    for mut font in &mut text_query {
        font.font = ui_font.0.clone();
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
    mut solution_state: ResMut<SolutionState>,
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
                if inventory.hotbar[slot.index].is_some() {
                    inventory.hotbar[slot.index] = None;
                    solution_state.dirty = true;
                }
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
            let before = inventory.hotbar;
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
            if inventory.hotbar != before {
                solution_state.dirty = true;
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
                *border = pressed_border();
            }
            Interaction::Hovered => {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            }
            Interaction::None => {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        }
    }
}

fn pause_action_visible(
    save_state: &SaveState,
    solution_state: &SolutionState,
    action: PauseAction,
) -> bool {
    match action {
        PauseAction::ToggleBuilderMode => solution_state.entry != WorldEntryMode::PlaySolution,
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
    config: Res<GameConfig>,
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
    mut simulation_status_text: Query<
        &mut Text,
        (
            With<SimulationStatusText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTitle>,
            Without<FovText>,
            Without<UiScaleText>,
            Without<SimulationText>,
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
    if let Ok(mut text) = hotbar.single_mut() {
        let selected_item = inventory.hotbar[placement.selected];
        let selected = selected_item
            .map(|item| i18n.text(item.name_key()))
            .unwrap_or_else(|| i18n.text("empty"));
        text.0 = i18n.fmt(
            "status.hotbar",
            &[
                ("mode", builder_mode_name(*builder_mode, &i18n)),
                ("selected", selected),
            ],
        );
    }

    if let Ok(mut text) = inventory_title.single_mut() {
        text.0 = i18n.fmt(
            "inventory.title",
            &[("mode", builder_mode_name(*builder_mode, &i18n))],
        );
    }

    if let Ok(mut text) = fov_text.single_mut() {
        text.0 = format!("FOV {:.0}", settings.fov_degrees);
    }

    if let Ok(mut text) = ui_scale_text.single_mut() {
        text.0 = i18n.fmt(
            "settings.ui_scale",
            &[("scale", format!("{:.1}", settings.ui_scale))],
        );
    }

    if let Ok(mut text) = simulation_text.single_mut() {
        text.0 = i18n.fmt(
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

    if let Ok(mut text) = simulation_status_text.single_mut() {
        text.0 = if *builder_mode == BuilderMode::Play {
            simulation_status_text_value(&simulation, &config, &i18n)
        } else {
            String::new()
        };
    }

    if let Ok(mut text) = current_save_text.single_mut() {
        text.0 = save_state
            .current
            .as_ref()
            .map(|name| i18n.fmt("save.world", &[("name", name.clone())]))
            .unwrap_or_else(|| i18n.text("save.no_world_loaded"));
    }
}

fn simulation_status_text_value(
    simulation: &SimulationState,
    config: &GameConfig,
    i18n: &I18n,
) -> String {
    let start = config.input(ConfigAction::Simulate).name().to_string();
    let fast = config
        .input(ConfigAction::SimulationFast)
        .name()
        .to_string();
    let step = config
        .input(ConfigAction::SimulationStep)
        .name()
        .to_string();
    let rollback = config
        .input(ConfigAction::SimulationRollback)
        .name()
        .to_string();

    let (state_key, controls_key, controls_args): (&str, &str, Vec<(&str, String)>) =
        if !simulation.is_active() {
            (
                "simulation_state.ready",
                "simulation_controls.ready",
                vec![("start", start)],
            )
        } else if simulation.running && simulation.speed > 1.0 {
            (
                "simulation_state.fast",
                "simulation_controls.fast",
                vec![("fast", fast), ("step", step), ("rollback", rollback)],
            )
        } else if simulation.running {
            (
                "simulation_state.playing",
                "simulation_controls.playing",
                vec![("step", step), ("fast", fast), ("rollback", rollback)],
            )
        } else {
            (
                "simulation_state.paused",
                "simulation_controls.paused",
                vec![("step", step), ("start", start), ("rollback", rollback)],
            )
        };
    let controls = i18n.fmt(controls_key, &controls_args);

    i18n.fmt(
        "status.simulation_overlay",
        &[
            ("state", i18n.text(state_key)),
            ("turns", simulation.turn.to_string()),
            ("controls", controls),
        ],
    )
}

pub fn update_carried_item_ui(
    carried: Res<CarriedItem>,
    i18n: Res<I18n>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut icon: Query<(&mut Node, &mut BackgroundColor, &Children), With<CarriedIcon>>,
    mut icon_images: Query<&mut ImageNode, Without<CarriedIcon>>,
    mut label: Query<&mut Text, With<CarriedLabel>>,
) {
    let Ok((mut style, mut background, children)) = icon.single_mut() else {
        return;
    };

    let Some(item) = carried.item() else {
        style.display = Display::None;
        if let Ok(mut text) = label.single_mut() {
            text.0.clear();
        }
        for child in children.iter() {
            if let Ok(mut image) = icon_images.get_mut(child) {
                *image = ImageNode::default();
            }
        }
        return;
    };

    let Ok(window) = windows.single() else {
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

    let icon_handle = item
        .block()
        .and_then(|kind| block_icons.as_deref().and_then(|icons| icons.get(kind)));
    for child in children.iter() {
        if let Ok(mut image) = icon_images.get_mut(child) {
            *image = icon_handle
                .as_ref()
                .map(|handle| ImageNode::new(handle.clone()))
                .unwrap_or_default();
        }
    }

    if let Ok(mut text) = label.single_mut() {
        text.0 = if icon_handle.is_some() {
            String::new()
        } else {
            i18n.text(short_item_name(item))
        };
    }
}

pub fn update_scroll_containers(
    ui_runtime: Res<UiRuntime>,
    _settings_tab: Res<SettingsTab>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut containers: Query<(&mut ScrollContainer, &Children, &ComputedNode)>,
    mut contents: Query<(&mut Node, &ComputedNode), With<ScrollContent>>,
) {
    if !ui_runtime.is_settings_open() {
        return;
    }

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();

    for (mut container, children, node) in &mut containers {
        let Some(child) = children.iter().find(|child| contents.get(*child).is_ok()) else {
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
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut generator_period_text: Query<&mut Text, With<GeneratorPeriodText>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let generator_settings = world.generator_settings(pos);
    if let Ok(mut text) = generator_period_text.single_mut() {
        text.0 = generator_settings.period.to_string();
    }
}

pub fn update_labeler_ui(
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    i18n: Res<I18n>,
    mut title_text: Query<&mut Text, With<super::types::LocalizedText>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let Some(block) = world.system_blocks.get(&pos) else {
        return;
    };
    let key = match block.kind {
        BlockKind::Stamper => "stamper.title",
        BlockKind::Roller => "roller.title",
        _ => "labeler.title",
    };
    for mut text in &mut title_text {
        if text.0 == i18n.text("labeler.title")
            || text.0 == i18n.text("stamper.title")
            || text.0 == i18n.text("roller.title")
        {
            text.0 = i18n.text(key);
        }
    }
}

pub fn update_converter_ui(mut converter_input_row: Query<&mut Node, With<ConverterInputRow>>) {
    for mut style in &mut converter_input_row {
        style.display = Display::Flex;
    }
}

pub fn update_teleport_ui(
    ui_runtime: Res<UiRuntime>,
    rename_state: Res<TeleportRenameState>,
    world: Res<WorldBlocks>,
    mut teleport_name_text: Query<&mut Text, With<TeleportNameText>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let settings = world.teleport_settings(pos);
    if let Ok(mut text) = teleport_name_text.single_mut() {
        text.0 = if rename_state.editing == Some(pos) {
            format!("{}_", rename_state.buffer)
        } else {
            settings.name
        };
    }
}

pub fn update_block_panel_dropdowns_ui(
    mut commands: Commands,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    i18n: Res<I18n>,
    mut teleport_pair_cache: Local<Option<(Option<IVec3>, u64, Language, bool)>>,
    mut dropdowns: BlockPanelDropdownParams,
) {
    let active_pos = ui_runtime.active_block_pos();

    for (label, mut text) in &mut dropdowns.labels {
        text.0 = match label.0 {
            BlockPanelDropdown::GeneratorMaterial => active_pos
                .map(|pos| world.generator_settings(pos).material)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::GoalMaterial => active_pos
                .map(|pos| world.goal_settings(pos).material)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::LabelerColor => active_pos
                .map(|pos| world.labeler_settings(pos).color)
                .map(|color| i18n.text(color.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::ConverterInput => active_pos
                .map(|pos| world.converter_settings(pos).input)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::ConverterOutput => active_pos
                .map(|pos| world.converter_settings(pos).output)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::TeleportPair => active_pos
                .and_then(|pos| world.teleport_settings(pos).pair)
                .map(|pair| world.teleport_settings(pair).name)
                .unwrap_or_else(|| i18n.text("teleport.none")),
        };
    }

    for (list, mut style) in &mut dropdowns.lists {
        style.display = if open_dropdown.0 == Some(list.0) {
            Display::Flex
        } else {
            Display::None
        };
    }

    let pair_dropdown_open = open_dropdown.0 == Some(BlockPanelDropdown::TeleportPair);
    let pair_cache_key = (
        active_pos,
        world.topology_revision,
        i18n.language(),
        pair_dropdown_open,
    );
    let rebuild_pair_options = *teleport_pair_cache != Some(pair_cache_key);
    if rebuild_pair_options {
        *teleport_pair_cache = Some(pair_cache_key);
    }

    for (entity, list, children) in &mut dropdowns.teleport_pair_list {
        if list.0 != BlockPanelDropdown::TeleportPair {
            continue;
        }
        if !rebuild_pair_options {
            continue;
        }
        if let Some(children) = children {
            for child in children {
                if dropdowns.teleport_pair_options.get(*child).is_ok() {
                    commands.entity(*child).despawn();
                }
            }
        }
        if pair_dropdown_open {
            let Some(pos) = active_pos else {
                continue;
            };
            commands.entity(entity).with_children(|parent| {
                spawn_teleport_pair_option(parent, i18n.text("teleport.none"), None);
                for pair in teleport_pair_candidates(&world, pos) {
                    spawn_teleport_pair_option(
                        parent,
                        world.teleport_settings(pair).name,
                        Some(pair),
                    );
                }
            });
        }
    }
}

fn spawn_teleport_pair_option(
    parent: &mut ChildSpawnerCommands,
    label: String,
    pair: Option<IVec3>,
) {
    parent
        .spawn((menu_button(32.0), TeleportAction::SetPair(pair)))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: super::components::default_font_size(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn teleport_pair_candidates(world: &WorldBlocks, pos: IVec3) -> Vec<IVec3> {
    let Some(block) = world.system_blocks.get(&pos) else {
        return Vec::new();
    };
    let target_kind = match block.kind {
        BlockKind::TeleportEntrance => BlockKind::TeleportExit,
        BlockKind::TeleportExit => BlockKind::TeleportEntrance,
        _ => return Vec::new(),
    };
    let mut candidates: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(candidate_pos, candidate)| {
            (candidate.kind == target_kind).then_some(*candidate_pos)
        })
        .collect();
    candidates.sort_by_key(|candidate| world.teleport_settings(*candidate).name);
    candidates
}

fn builder_mode_name(mode: BuilderMode, i18n: &I18n) -> String {
    match mode {
        BuilderMode::Edit => i18n.text("mode.edit"),
        BuilderMode::Play => i18n.text("mode.play"),
    }
}

pub fn update_settings_text_ui(
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    i18n: Res<I18n>,
    mut key_labels: Query<
        (&ChildOf, &mut Text),
        (
            With<KeyBindingLabel>,
            Without<SettingsDropdownLabel>,
            Without<SettingsValueText>,
        ),
    >,
    mut place_mode_text: Query<
        &mut Text,
        (
            With<PlaceSelectionModeText>,
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
            Without<KeyBindingLabel>,
            Without<PlaceSelectionModeText>,
            Without<SettingsDropdownLabel>,
            Without<SettingsValueText>,
        ),
    >,
    key_buttons: Query<&KeyBindingButton>,
) {
    for (parent, mut text) in &mut key_labels {
        let Ok(button) = key_buttons.get(parent.parent()) else {
            continue;
        };
        let suffix = pending_key_bind
            .0
            .filter(|pending| *pending == button.0)
            .map(|_| "...")
            .unwrap_or(config.input(button.0).name());
        text.0 = format!("{}: {suffix}", i18n.text(button.0.label_key()));
    }

    if let Ok(mut text) = place_mode_text.single_mut() {
        text.0 = i18n.text(config.place_selection_mode.label_key());
    }

    if let Ok(mut text) = delete_mode_text.single_mut() {
        text.0 = i18n.text(config.delete_selection_mode.label_key());
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
        if let Some(slider) = settings_action_slider(*action) {
            if active_slider.0 == Some(slider) {
                continue;
            }
            let next_value = settings_slider_percent(slider, &settings);
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
        let Some(slider) = settings_action_slider(*action) else {
            continue;
        };
        let percent = (range.thumb_position(value.0) * 100.0).clamp(0.0, 100.0);

        for (fill, mut style) in &mut slider_fills {
            if fill.0 == slider {
                style.width = Val::Percent(percent);
            }
        }

        for (knob, mut style) in &mut slider_knobs {
            if knob.0 == slider {
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
    mut dropdown_labels: Query<
        (&SettingsDropdownLabel, &mut Text),
        (
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
            Without<KeyBindingLabel>,
            Without<FovText>,
            Without<PlaceSelectionModeText>,
            Without<DeleteSelectionModeText>,
            Without<SettingsDropdownLabel>,
        ),
    >,
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
    for (label, mut text) in &mut dropdown_labels {
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

    for (value, mut text) in &mut value_texts {
        text.0 = match value.0 {
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
            *background = Color::srgb(0.56, 0.56, 0.56).into();
            *border = pressed_border();
        } else if matches!(
            *action,
            SettingsAction::TabGameplay | SettingsAction::TabKeyBindings
        ) {
            if *interaction == Interaction::Hovered {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            } else {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        } else {
            match *interaction {
                Interaction::Pressed => {
                    *background = BUTTON_PRESSED_BG.into();
                    *border = pressed_border();
                }
                Interaction::Hovered => {
                    *background = BUTTON_HOVER_BG.into();
                    *border = hover_border();
                }
                Interaction::None => {
                    *background = BUTTON_BG.into();
                    *border = raised_border();
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

fn live_slider_percent(
    slider: SettingsSlider,
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
            ((drag_state.dragging || active_slider.0 == Some(slider))
                && settings_action_slider(*action) == Some(slider))
            .then_some(value.0.clamp(0.0, 100.0))
        })
        .unwrap_or_else(|| settings_slider_percent(slider, settings))
}

fn settings_action_slider(action: SettingsAction) -> Option<SettingsSlider> {
    match action {
        SettingsAction::FovSlider => Some(SettingsSlider::Fov),
        SettingsAction::UiScaleSlider => Some(SettingsSlider::UiScale),
        SettingsAction::GravitySlider => Some(SettingsSlider::Gravity),
        _ => None,
    }
}

pub fn update_localized_ui(
    i18n: Res<I18n>,
    save_state: Res<SaveState>,
    mut localized_text: Query<(&LocalizedText, &mut Text)>,
) {
    if !i18n.is_changed() && !save_state.is_changed() {
        return;
    }

    for (localized, mut text) in &mut localized_text {
        text.0 = if localized.key == "button.save_world" {
            match save_state.current_kind {
                Some(SaveKind::Solution) => i18n.text("button.save_solution"),
                _ => i18n.text("button.save_puzzle"),
            }
        } else {
            i18n.text(localized.key)
        };
    }
}

pub fn update_panel_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    settings_tab: Res<SettingsTab>,
    ui_runtime: Res<UiRuntime>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut style_sets: ParamSet<(
        Query<&mut Node, (With<MainMenuPanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<SaveListPanel>, Without<PauseAction>)>,
        Query<
            &mut Node,
            (
                With<SettingsGameplayGroup>,
                Without<PauseAction>,
                Without<UiPanelBinding>,
            ),
        >,
        Query<
            &mut Node,
            (
                With<SettingsKeyBindingsGroup>,
                Without<PauseAction>,
                Without<UiPanelBinding>,
            ),
        >,
        Query<&mut Node, (With<BackpackPanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<PausePanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<ConfirmDialogPanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<ModalScrim>, Without<PauseAction>)>,
    )>,
    mut pause_buttons: Query<(&PauseAction, &mut Node), With<Button>>,
    mut bound_panels: Query<
        (&UiPanelBinding, &mut Node),
        (
            Without<PauseAction>,
            Without<MainMenuPanel>,
            Without<SaveListPanel>,
            Without<SettingsGameplayGroup>,
            Without<SettingsKeyBindingsGroup>,
            Without<BackpackPanel>,
            Without<PausePanel>,
            Without<ConfirmDialogPanel>,
            Without<ModalScrim>,
        ),
    >,
    confirm_dialog: Res<ConfirmDialogState>,
) {
    for mut style in &mut style_sets.p0() {
        style.display = if *mode == GameMode::MainMenu {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p1() {
        style.display = if *mode == GameMode::SaveListMain {
            Display::Flex
        } else {
            Display::None
        };
    }

    let settings_open = ui_runtime.is_settings_open();

    for mut style in &mut style_sets.p2() {
        style.display = if settings_open && *settings_tab == SettingsTab::Gameplay {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p3() {
        style.display = if settings_open && *settings_tab == SettingsTab::KeyBindings {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p4() {
        style.display = if *mode == GameMode::Inventory {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p5() {
        style.display = if *mode == GameMode::Paused {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p6() {
        style.display = if confirm_dialog.kind.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }

    let active_panel = ui_runtime.active_panel();
    for mut style in &mut style_sets.p7() {
        style.display = if ui_runtime.has_modal_panel() || confirm_dialog.kind.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (action, mut style) in &mut pause_buttons {
        style.display = if pause_action_visible(&save_state, &solution_state, *action) {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !block_dropdown_matches_panel(open_block_dropdown.0, active_panel) {
        open_block_dropdown.0 = None;
    }
    for (binding, mut style) in &mut bound_panels {
        let visible = active_panel == Some(binding.0);
        style.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_ui_layers(
    ui_runtime: Res<UiRuntime>,
    confirm_dialog: Res<ConfirmDialogState>,
    mut layered_nodes: Query<(
        &mut GlobalZIndex,
        Option<&UiPanelBinding>,
        Has<MainMenuPanel>,
        Has<SaveListPanel>,
        Has<PausePanel>,
        Has<BackpackPanel>,
        Has<ConfirmDialogPanel>,
        Has<ModalScrim>,
    )>,
) {
    const BASE_LAYER: i32 = 100;

    let top_panel_z = ui_runtime
        .top_modal_layer()
        .map(panel_layer_z)
        .unwrap_or(PANEL_LAYER_BASE);
    let confirm_z = if confirm_dialog.kind.is_some() {
        top_panel_z + CONFIRM_LAYER_STEP
    } else {
        PANEL_LAYER_BASE
    };
    let scrim_z = if confirm_dialog.kind.is_some() {
        confirm_z + SCRIM_OFFSET
    } else {
        ui_runtime
            .top_modal_layer()
            .map(|layer| panel_layer_z(layer) + SCRIM_OFFSET)
            .unwrap_or(PANEL_LAYER_BASE + SCRIM_OFFSET)
    };

    for (
        mut z,
        binding,
        main_menu,
        save_list,
        pause_panel,
        backpack_panel,
        confirm_panel,
        modal_scrim,
    ) in &mut layered_nodes
    {
        z.0 = if modal_scrim {
            scrim_z
        } else if confirm_panel {
            confirm_z
        } else if let Some(binding) = binding {
            ui_runtime
                .panel_layer(binding.0)
                .map(panel_layer_z)
                .unwrap_or(PANEL_LAYER_BASE)
        } else if main_menu || save_list || pause_panel || backpack_panel {
            BASE_LAYER
        } else {
            z.0
        };
    }
}

const PANEL_LAYER_BASE: i32 = 1_000;
const PANEL_LAYER_STEP: i32 = 20;
const SCRIM_OFFSET: i32 = -1;
const CONFIRM_LAYER_STEP: i32 = 20;

fn panel_layer_z(layer: usize) -> i32 {
    PANEL_LAYER_BASE + layer as i32 * PANEL_LAYER_STEP
}

fn block_dropdown_matches_panel(
    dropdown: Option<BlockPanelDropdown>,
    panel: Option<UiPanelId>,
) -> bool {
    matches!(
        (dropdown, panel),
        (None, _)
            | (
                Some(BlockPanelDropdown::GeneratorMaterial),
                Some(UiPanelId::Generator)
            )
            | (
                Some(BlockPanelDropdown::GoalMaterial),
                Some(UiPanelId::Goal)
            )
            | (
                Some(BlockPanelDropdown::LabelerColor),
                Some(UiPanelId::Labeler)
            )
            | (
                Some(BlockPanelDropdown::ConverterInput),
                Some(UiPanelId::Converter)
            )
            | (
                Some(BlockPanelDropdown::ConverterOutput),
                Some(UiPanelId::Converter)
            )
            | (
                Some(BlockPanelDropdown::TeleportPair),
                Some(UiPanelId::Teleport)
            )
    )
}

pub fn update_hud_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    mut hud_style: Query<&mut Node, With<InGameHudStyle>>,
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
    block_icons: Option<Res<BlockIconAssets>>,
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
    mut labels: Query<
        &mut Text,
        (
            With<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<InventoryTooltipText>,
        ),
    >,
    mut icons: Query<&mut ImageNode, With<SlotIcon>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tooltip: Query<&mut Node, (With<InventoryTooltip>, Without<SlotIcon>)>,
    mut tooltip_text: Query<&mut Text, (With<InventoryTooltipText>, Without<SlotLabel>)>,
) {
    let mut hovered_item = None;
    for (slot, interaction, children, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };
        let icon_handle = item
            .and_then(|item| item.block())
            .and_then(|kind| block_icons.as_deref().and_then(|icons| icons.get(kind)));
        let has_icon = icon_handle.is_some();
        if *interaction == Interaction::Hovered {
            hovered_item = item;
        }

        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;
        let base_color = item
            .map(slot_color)
            .unwrap_or(Color::srgb(0.255, 0.251, 0.251));
        *background = if has_icon && *interaction == Interaction::Hovered {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else if has_icon {
            Color::srgb(0.255, 0.251, 0.251).into()
        } else if *interaction == Interaction::Hovered && item.is_none() {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else if *interaction == Interaction::Hovered {
            base_color.with_alpha(1.0).into()
        } else {
            base_color.into()
        };
        *border = if selected_hotbar {
            BorderColor {
                top: Color::srgb(1.0, 0.94, 0.80),
                left: Color::srgb(1.0, 0.94, 0.80),
                right: Color::srgb(0.36, 0.25, 0.12),
                bottom: Color::srgb(0.36, 0.25, 0.12),
            }
        } else if *interaction == Interaction::Hovered {
            hover_border()
        } else {
            inset_border()
        };

        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = if has_icon {
                    String::new()
                } else {
                    item.map(|kind| i18n.text(short_item_name(kind)))
                        .unwrap_or_default()
                };
            }
            if let Ok(mut image) = icons.get_mut(child) {
                *image = icon_handle.clone().map(ImageNode::new).unwrap_or_default();
            }
        }
    }

    let Ok(mut tooltip_node) = tooltip.single_mut() else {
        return;
    };
    let Some(item) = hovered_item else {
        tooltip_node.display = Display::None;
        return;
    };
    let Ok(window) = windows.single() else {
        tooltip_node.display = Display::None;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        tooltip_node.display = Display::None;
        return;
    };

    tooltip_node.display = Display::Flex;
    tooltip_node.left = Val::Px(cursor.x + 16.0);
    tooltip_node.top = Val::Px(cursor.y + 16.0);
    if let Ok(mut text) = tooltip_text.single_mut() {
        text.0 = i18n.text(item.name_key());
    }
}

pub fn update_save_list_ui(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
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
    if let Ok(mut title) = text_sets.p0().single_mut() {
        title.0 = match *mode {
            GameMode::SaveListMain => i18n.text("save.title.main"),
            _ => i18n.text("save.title.default"),
        };
    }

    let puzzles = save_state.puzzles();
    let solutions = save_state
        .selected_puzzle
        .as_deref()
        .map(|puzzle| save_state.solutions_for_puzzle(puzzle))
        .unwrap_or_default();
    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;

    for (action, interaction, children, mut background) in &mut slots {
        let label = match *action {
            SaveListAction::LoadPuzzle(index) => puzzles
                .get(index)
                .map(|entry| {
                    if save_state.selected_puzzle.as_deref() == Some(entry.name.as_str()) {
                        i18n.fmt("save.selected_puzzle", &[("name", entry.name.clone())])
                    } else if play_flow {
                        i18n.fmt("save.select_puzzle", &[("name", entry.name.clone())])
                    } else {
                        i18n.fmt("save.load_puzzle", &[("name", entry.name.clone())])
                    }
                })
                .unwrap_or_else(|| i18n.text("empty_slot")),
            SaveListAction::LoadSolution(index) => solutions
                .get(index)
                .map(|entry| i18n.fmt("save.load_solution", &[("name", entry.name.clone())]))
                .unwrap_or_else(|| i18n.text("empty_slot")),
            SaveListAction::DeletePuzzle(_) | SaveListAction::DeleteSolution(_) => {
                i18n.text("button.delete")
            }
            SaveListAction::NewPuzzle => i18n.text("button.new_puzzle"),
            SaveListAction::NewSolution => i18n.text("button.new_solution"),
            SaveListAction::Back => i18n.text("button.back"),
        };

        let enabled_load = match *action {
            SaveListAction::LoadPuzzle(index) => puzzles.get(index).is_some(),
            SaveListAction::LoadSolution(index) => play_flow && solutions.get(index).is_some(),
            SaveListAction::DeletePuzzle(index) => puzzles.get(index).is_some(),
            SaveListAction::DeleteSolution(index) => play_flow && solutions.get(index).is_some(),
            SaveListAction::NewPuzzle => edit_flow,
            SaveListAction::NewSolution => play_flow && save_state.selected_puzzle.is_some(),
            SaveListAction::Back => true,
        };
        let selected_puzzle_button = matches!(*action, SaveListAction::LoadPuzzle(_))
            && match *action {
                SaveListAction::LoadPuzzle(index) => puzzles.get(index).is_some_and(|entry| {
                    save_state.selected_puzzle.as_deref() == Some(entry.name.as_str())
                }),
                _ => false,
            };

        *background = if enabled_load && *interaction == Interaction::Hovered {
            BUTTON_HOVER_BG.into()
        } else if enabled_load && selected_puzzle_button {
            Color::srgba(0.22, 0.35, 0.32, 0.96).into()
        } else if enabled_load {
            BUTTON_BG.into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = text_sets.p1().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

pub fn update_confirm_dialog_ui(
    dialog: Res<ConfirmDialogState>,
    i18n: Res<I18n>,
    mut title: Query<&mut Text, With<ConfirmDialogTitle>>,
    mut message: Query<&mut Text, (With<ConfirmDialogMessage>, Without<ConfirmDialogTitle>)>,
    mut primary: Query<
        &mut Text,
        (
            With<ConfirmDialogPrimaryLabel>,
            Without<ConfirmDialogTitle>,
            Without<ConfirmDialogMessage>,
        ),
    >,
    mut secondary: Query<
        &mut Text,
        (
            With<ConfirmDialogSecondaryLabel>,
            Without<ConfirmDialogTitle>,
            Without<ConfirmDialogMessage>,
            Without<ConfirmDialogPrimaryLabel>,
        ),
    >,
    mut action_buttons: Query<(&ConfirmDialogAction, &mut Node), With<Button>>,
) {
    if !dialog.is_changed() && !i18n.is_changed() {
        return;
    }

    let Some(kind) = dialog.kind.as_ref() else {
        return;
    };
    if let Ok(mut text) = title.single_mut() {
        text.0 = i18n.text("confirm.title");
    }
    if let Ok(mut text) = message.single_mut() {
        text.0 = match kind {
            ConfirmDialogKind::DeleteSave { name } => {
                i18n.fmt("save.confirm_delete", &[("name", name.clone())])
            }
            ConfirmDialogKind::ResetSolution => i18n.text("confirm.reset_solution"),
            ConfirmDialogKind::ReturnToMain => i18n.text("confirm.return_to_main"),
            ConfirmDialogKind::SaveSolutionBeforeEdit => {
                i18n.text("confirm.save_solution_before_edit")
            }
        };
    }
    if let Ok(mut text) = primary.single_mut() {
        text.0 = match kind {
            ConfirmDialogKind::DeleteSave { .. } => i18n.text("button.delete"),
            ConfirmDialogKind::ResetSolution => i18n.text("button.confirm_reset_solution"),
            ConfirmDialogKind::ReturnToMain => i18n.text("button.save_and_back"),
            ConfirmDialogKind::SaveSolutionBeforeEdit => i18n.text("button.save_solution_and_edit"),
        };
    }
    if let Ok(mut text) = secondary.single_mut() {
        text.0 = match kind {
            ConfirmDialogKind::ReturnToMain => i18n.text("button.discard_and_back"),
            ConfirmDialogKind::SaveSolutionBeforeEdit => {
                i18n.text("button.discard_solution_and_edit")
            }
            _ => String::new(),
        };
    }

    let secondary_visible = matches!(
        kind,
        ConfirmDialogKind::ReturnToMain | ConfirmDialogKind::SaveSolutionBeforeEdit
    );
    for (action, mut node) in &mut action_buttons {
        if matches!(*action, ConfirmDialogAction::Secondary) {
            node.display = if secondary_visible {
                Display::Flex
            } else {
                Display::None
            };
        } else {
            node.display = Display::Flex;
        }
    }
}
