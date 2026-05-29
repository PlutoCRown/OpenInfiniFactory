use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SettingsReturnMode, SimulationState,
    SolutionState, TeleportRenameState,
};
use crate::game::ui::{
    CarriedItem, ConverterAction, GeneratorAction, InventoryItems, LabelerAction, MainMenuAction,
    OpenSettingsDropdown, PauseAction, PendingKeyBind, SaveListAction, SettingsAction, SettingsTab,
    TeleportAction,
};
use crate::game::world::blocks::{MaterialKind, StampColor};
use crate::game::world::grid::{seed_demo_world, WorldBlocks};
use crate::game::world::rendering::{despawn_world, rebuild_world, BlockEntity, WorldRenderAssets};
use crate::game::{UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{input_from_buttons, open_config_folder, save_config, GameConfig};
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::{
    load_world, next_world_name, reset_solution_world, save_solution_with_puzzle, save_world,
    SaveKind, SaveState,
};

pub fn main_menu_actions(
    mut exit: EventWriter<AppExit>,
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut settings_return: ResMut<SettingsReturnMode>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    render_assets: Res<WorldRenderAssets>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut interactions: Query<(&Interaction, &MainMenuAction), (Changed<Interaction>, With<Button>)>,
) {
    if *mode != GameMode::MainMenu {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            MainMenuAction::NewWorld => {
                let name = next_world_name(&save_state.slots);
                world.clear();
                seed_demo_world(&mut world);
                save_world(&world, &name, SaveKind::Puzzle);
                save_state.current = Some(name);
                save_state.current_kind = Some(SaveKind::Puzzle);
                solution_state.puzzle_snapshot = None;
                save_state.refresh();
                reset_builder_state(
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                );
                despawn_world(&mut commands, &block_entities);
                rebuild_world(&mut commands, &world, &render_assets);
                *mode = GameMode::Playing;
            }
            MainMenuAction::OpenSaveList => {
                save_state.refresh();
                *mode = GameMode::SaveListMain;
            }
            MainMenuAction::OpenSettings => {
                settings_return.0 = GameMode::MainMenu;
                *mode = GameMode::Settings;
            }
            MainMenuAction::Quit => {
                exit.send(AppExit::Success);
            }
        }
    }
}

pub fn save_list_actions(
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut simulation: ResMut<SimulationState>,
    render_assets: Res<WorldRenderAssets>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut interactions: Query<(&Interaction, &SaveListAction), (Changed<Interaction>, With<Button>)>,
) {
    if !matches!(*mode, GameMode::SaveListMain | GameMode::SaveListPause) {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            SaveListAction::Load(index) => {
                let Some(name) = save_state.slots.get(index).cloned() else {
                    continue;
                };
                if let Some(mut loaded) = load_world(&mut world, &name) {
                    let opening_from_play = *mode == GameMode::SaveListPause;
                    if opening_from_play && loaded.kind == SaveKind::Puzzle {
                        loaded.puzzle_snapshot = Some(loaded.world.clone());
                        loaded.kind = SaveKind::Solution;
                    }
                    save_state.current = Some(name);
                    save_state.current_kind = Some(loaded.kind);
                    solution_state.puzzle_snapshot = loaded.puzzle_snapshot;
                    simulation.running = false;
                    simulation.turn = 0;
                    simulation.accumulator = 0.0;
                    simulation.start_snapshot = None;
                    if opening_from_play {
                        *builder_mode = BuilderMode::Play;
                        *inventory = InventoryItems::for_mode(*builder_mode);
                        carried.clear();
                        placement.selected = 0;
                    } else {
                        reset_builder_state(
                            &mut builder_mode,
                            &mut inventory,
                            &mut carried,
                            &mut placement,
                        );
                    }
                    despawn_world(&mut commands, &block_entities);
                    rebuild_world(&mut commands, &world, &render_assets);
                    *mode = GameMode::Playing;
                }
            }
            SaveListAction::Back => {
                *mode = match *mode {
                    GameMode::SaveListPause => GameMode::Paused,
                    _ => GameMode::MainMenu,
                };
            }
        }
    }
}

pub fn pause_menu_actions(
    mut exit: EventWriter<AppExit>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut settings_return: ResMut<SettingsReturnMode>,
    mut world: ResMut<WorldBlocks>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    render_assets: Res<WorldRenderAssets>,
    mut interactions: Query<(&Interaction, &PauseAction), (Changed<Interaction>, With<Button>)>,
) {
    if !matches!(*mode, GameMode::Paused | GameMode::ConfirmSaveSolutionBeforeEdit) {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            PauseAction::Resume => *mode = GameMode::Playing,
            PauseAction::ToggleBuilderMode => {
                *builder_mode = match *builder_mode {
                    BuilderMode::Edit => {
                        simulation.running = false;
                        simulation.accumulator = 0.0;
                        simulation.start_snapshot = None;
                        if save_state.current_kind == Some(SaveKind::Puzzle) {
                            solution_state.puzzle_snapshot = Some(world.clone());
                        }
                        save_state.current_kind = Some(SaveKind::Solution);
                        BuilderMode::Play
                    }
                    BuilderMode::Play => {
                        *mode = GameMode::ConfirmSaveSolutionBeforeEdit;
                        continue;
                    }
                };
                *inventory = InventoryItems::for_mode(*builder_mode);
                carried.clear();
                placement.selected = 0;
            }
            PauseAction::ConfirmSaveSolutionAndEdit => {
                save_current_world(&world, &mut save_state, &mut solution_state);
                switch_to_edit_mode(
                    &mut world,
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut mode,
                    &mut save_state,
                    &mut solution_state,
                );
                despawn_world(&mut commands, &block_entities);
                rebuild_world(&mut commands, &world, &render_assets);
            }
            PauseAction::DiscardSolutionAndEdit => {
                switch_to_edit_mode(
                    &mut world,
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut mode,
                    &mut save_state,
                    &mut solution_state,
                );
                despawn_world(&mut commands, &block_entities);
                rebuild_world(&mut commands, &world, &render_assets);
            }
            PauseAction::CancelEditSwitch => {
                *mode = GameMode::Paused;
            }
            PauseAction::SaveWorld => {
                save_current_world(&world, &mut save_state, &mut solution_state);
            }
            PauseAction::ResetSolution => {
                if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
                    reset_solution_world(&mut world, puzzle_snapshot);
                    simulation.running = false;
                    simulation.turn = 0;
                    simulation.accumulator = 0.0;
                    simulation.start_snapshot = None;
                    despawn_world(&mut commands, &block_entities);
                    rebuild_world(&mut commands, &world, &render_assets);
                }
            }
            PauseAction::OpenSaveList => {
                save_state.refresh();
                *mode = GameMode::SaveListPause;
            }
            PauseAction::OpenSettings => {
                settings_return.0 = GameMode::Paused;
                *mode = GameMode::Settings;
            }
            PauseAction::BackToMainMenu => {
                simulation.running = false;
                simulation.accumulator = 0.0;
                simulation.start_snapshot = None;
                placement.generator_panel = None;
                world.clear();
                save_state.current = None;
                save_state.current_kind = None;
                solution_state.puzzle_snapshot = None;
                despawn_world(&mut commands, &block_entities);
                rebuild_world(&mut commands, &world, &render_assets);
                *mode = GameMode::MainMenu;
            }
            PauseAction::Quit => {
                exit.send(AppExit::Success);
            }
        }
    }
}

fn save_current_world(
    world: &WorldBlocks,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
) {
    let kind = save_state.current_kind.unwrap_or(SaveKind::Puzzle);
    let name = save_state
        .current
        .clone()
        .unwrap_or_else(|| next_world_name(&save_state.slots));
    let saved = match kind {
        SaveKind::Puzzle => save_world(world, &name, SaveKind::Puzzle),
        SaveKind::Solution => {
            if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
                save_solution_with_puzzle(world, &name, puzzle_snapshot)
            } else {
                save_world(world, &name, SaveKind::Solution)
            }
        }
    };
    if saved {
        save_state.current = Some(name);
        save_state.current_kind = Some(kind);
        save_state.refresh();
    }
}

fn switch_to_edit_mode(
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    mode: &mut GameMode,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
) {
    if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
        *world = puzzle_snapshot.clone();
    }
    *builder_mode = BuilderMode::Edit;
    *inventory = InventoryItems::for_mode(*builder_mode);
    carried.clear();
    placement.selected = 0;
    save_state.current = None;
    save_state.current_kind = Some(SaveKind::Puzzle);
    solution_state.puzzle_snapshot = None;
    *mode = GameMode::Paused;
}

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mode: ResMut<GameMode>,
    settings_return: Res<SettingsReturnMode>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut i18n: ResMut<I18n>,
    mut settings_tab: ResMut<SettingsTab>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut interactions: Query<
        (Ref<Interaction>, &SettingsAction, &Node, &GlobalTransform),
        With<Button>,
    >,
) {
    if *mode != GameMode::Settings {
        pending_key_bind.0 = None;
        open_dropdown.0 = None;
        return;
    }

    if let Some(action) = pending_key_bind.0 {
        if let Some(input) = input_from_buttons(&keys, &mouse_buttons) {
            config.set_input(action, input);
            save_config(&config);
            pending_key_bind.0 = None;
        }
    }

    let cursor_position = windows
        .get_single()
        .ok()
        .and_then(|window| {
            window
                .cursor_position()
                .map(|cursor| Vec2::new(cursor.x - window.width() * 0.5, cursor.y))
        });

    for (interaction, action, node, transform) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            SettingsAction::TabGameplay => {
                if !interaction.is_changed() {
                    continue;
                }
                *settings_tab = SettingsTab::Gameplay;
                open_dropdown.0 = None;
            }
            SettingsAction::TabKeyBindings => {
                if !interaction.is_changed() {
                    continue;
                }
                *settings_tab = SettingsTab::KeyBindings;
                open_dropdown.0 = None;
            }
            SettingsAction::FovSlider => {
                if let Some(percent) = slider_percent(cursor_position, node, transform) {
                    settings.fov_degrees = (50.0 + percent * 60.0).round().clamp(50.0, 110.0);
                    config.fov_degrees = settings.fov_degrees;
                    save_config(&config);
                }
            }
            SettingsAction::UiScaleSlider => {
                if let Some(percent) = slider_percent(cursor_position, node, transform) {
                    settings.ui_scale =
                        ((UI_SCALE_MIN + percent * (UI_SCALE_MAX - UI_SCALE_MIN)) * 10.0).round()
                            / 10.0;
                    settings.ui_scale = settings.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
                    ui_scale.0 = settings.ui_scale;
                    config.ui_scale = settings.ui_scale;
                    save_config(&config);
                }
            }
            SettingsAction::SetPlaceSelectionMode(selection_mode) => {
                if !interaction.is_changed() {
                    continue;
                }
                config.place_selection_mode = selection_mode;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetDeleteSelectionMode(selection_mode) => {
                if !interaction.is_changed() {
                    continue;
                }
                config.delete_selection_mode = selection_mode;
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::SetLanguage(language) => {
                if !interaction.is_changed() {
                    continue;
                }
                i18n.set_language(language);
                config.language = Some(language);
                open_dropdown.0 = None;
                save_config(&config);
            }
            SettingsAction::ToggleDropdown(dropdown) => {
                if !interaction.is_changed() {
                    continue;
                }
                open_dropdown.0 = if open_dropdown.0 == Some(dropdown) {
                    None
                } else {
                    Some(dropdown)
                };
            }
            SettingsAction::Bind(action) => {
                if !interaction.is_changed() {
                    continue;
                }
                pending_key_bind.0 = Some(action);
            }
            SettingsAction::ResetDefaults => {
                if !interaction.is_changed() {
                    continue;
                }
                *config = GameConfig::default();
                settings.fov_degrees = config.fov_degrees;
                settings.ui_scale = config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
                ui_scale.0 = settings.ui_scale;
                i18n.set_language(resolve_language(config.language));
                open_dropdown.0 = None;
                pending_key_bind.0 = None;
                save_config(&config);
            }
            SettingsAction::OpenFolder => {
                if !interaction.is_changed() {
                    continue;
                }
                open_config_folder();
            }
            SettingsAction::Back => {
                if !interaction.is_changed() {
                    continue;
                }
                open_dropdown.0 = None;
                pending_key_bind.0 = None;
                *mode = settings_return.0;
            }
        }
    }
}

fn slider_percent(
    cursor_position: Option<Vec2>,
    node: &Node,
    transform: &GlobalTransform,
) -> Option<f32> {
    let cursor_position = cursor_position?;
    let width = node.size().x;
    if width <= 0.0 {
        return None;
    }
    let left = transform.translation().x - width * 0.5;
    Some(((cursor_position.x - left) / width).clamp(0.0, 1.0))
}

pub fn generator_menu_actions(
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
    mut world: ResMut<WorldBlocks>,
    mut interactions: Query<(&Interaction, &GeneratorAction), (Changed<Interaction>, With<Button>)>,
) {
    if *mode != GameMode::GeneratorSettings {
        return;
    }

    let Some(pos) = placement.generator_panel else {
        *mode = GameMode::Playing;
        return;
    };

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut settings = world.generator_settings(pos);
        match *action {
            GeneratorAction::PeriodDown => {
                settings.period = settings.period.saturating_sub(1).max(1);
                world.set_generator_settings(pos, settings);
            }
            GeneratorAction::PeriodUp => {
                settings.period = (settings.period + 1).min(120);
                world.set_generator_settings(pos, settings);
            }
            GeneratorAction::MaterialNext => {
                settings.material = next_material(settings.material);
                world.set_generator_settings(pos, settings);
            }
            GeneratorAction::Close => {
                placement.generator_panel = None;
                *mode = GameMode::Playing;
            }
        }
    }
}

pub fn labeler_menu_actions(
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
    mut world: ResMut<WorldBlocks>,
    mut interactions: Query<(&Interaction, &LabelerAction), (Changed<Interaction>, With<Button>)>,
) {
    if *mode != GameMode::LabelerSettings {
        return;
    }

    let Some(pos) = placement.labeler_panel else {
        *mode = GameMode::Playing;
        return;
    };

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut settings = world.labeler_settings(pos);
        match *action {
            LabelerAction::PreviousColor => {
                settings.color = previous_stamp_color(settings.color);
                world.set_labeler_settings(pos, settings);
            }
            LabelerAction::NextColor => {
                settings.color = next_stamp_color(settings.color);
                world.set_labeler_settings(pos, settings);
            }
            LabelerAction::Close => {
                placement.labeler_panel = None;
                *mode = GameMode::Playing;
            }
        }
    }
}

pub fn converter_menu_actions(
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
    mut world: ResMut<WorldBlocks>,
    mut interactions: Query<
        (&Interaction, &ConverterAction),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if *mode != GameMode::ConverterSettings {
        return;
    }

    let Some(pos) = placement.converter_panel else {
        *mode = GameMode::Playing;
        return;
    };

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut settings = world.converter_settings(pos);
        match *action {
            ConverterAction::ToggleMode => {
                settings.mode = settings.mode.toggle();
                world.set_converter_settings(pos, settings);
            }
            ConverterAction::InputNext => {
                settings.input = next_material(settings.input);
                world.set_converter_settings(pos, settings);
            }
            ConverterAction::OutputNext => {
                settings.output = next_material(settings.output);
                world.set_converter_settings(pos, settings);
            }
            ConverterAction::Close => {
                placement.converter_panel = None;
                *mode = GameMode::Playing;
            }
        }
    }
}

pub fn teleport_menu_actions(
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut interactions: Query<(&Interaction, &TeleportAction), (Changed<Interaction>, With<Button>)>,
) {
    if *mode != GameMode::TeleportSettings {
        return;
    }

    let Some(pos) = placement.teleport_panel else {
        *mode = GameMode::Playing;
        return;
    };

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            TeleportAction::CyclePair => world.cycle_teleport_pair(pos),
            TeleportAction::Rename => {
                let settings = world.teleport_settings(pos);
                rename_state.editing = Some(pos);
                rename_state.buffer = settings.name;
            }
            TeleportAction::Close => {
                rename_state.editing = None;
                placement.teleport_panel = None;
                *mode = GameMode::Playing;
            }
        }
    }
}

pub fn teleport_rename_input(
    mode: Res<GameMode>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut keyboard_input: EventReader<KeyboardInput>,
) {
    if *mode != GameMode::TeleportSettings || rename_state.editing.is_none() {
        keyboard_input.clear();
        return;
    }

    let pos = rename_state.editing.expect("checked above");
    let mut confirm = false;
    let mut cancel = false;

    for event in keyboard_input.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Enter => confirm = true,
            Key::Escape => cancel = true,
            Key::Backspace => {
                rename_state.buffer.pop();
            }
            Key::Space => push_rename_char(&mut rename_state.buffer, ' '),
            Key::Character(text) => {
                for ch in text.chars() {
                    push_rename_char(&mut rename_state.buffer, ch);
                }
            }
            _ => {}
        }
    }

    if confirm {
        let mut settings = world.teleport_settings(pos);
        let trimmed = rename_state.buffer.trim();
        if !trimmed.is_empty() {
            settings.name = trimmed.chars().take(24).collect();
            world.set_teleport_settings(pos, settings);
        }
        rename_state.editing = None;
    } else if cancel {
        rename_state.editing = None;
    }
}

fn push_rename_char(buffer: &mut String, ch: char) {
    if buffer.chars().count() >= 24 || ch.is_control() {
        return;
    }
    buffer.push(ch);
}

fn next_material(material: MaterialKind) -> MaterialKind {
    let all = MaterialKind::ALL;
    let index = all
        .iter()
        .position(|candidate| *candidate == material)
        .unwrap_or(0);
    all[(index + 1) % all.len()]
}

fn next_stamp_color(color: StampColor) -> StampColor {
    let all = StampColor::ALL;
    let index = all
        .iter()
        .position(|candidate| *candidate == color)
        .unwrap_or(0);
    all[(index + 1) % all.len()]
}

fn previous_stamp_color(color: StampColor) -> StampColor {
    let all = StampColor::ALL;
    let index = all
        .iter()
        .position(|candidate| *candidate == color)
        .unwrap_or(0);
    all[(index + all.len() - 1) % all.len()]
}

fn reset_builder_state(
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
) {
    *builder_mode = BuilderMode::Edit;
    *inventory = InventoryItems::for_mode(*builder_mode);
    carried.clear();
    placement.selected = 0;
    placement.edit_gesture = None;
    placement.generator_panel = None;
    placement.labeler_panel = None;
    placement.converter_panel = None;
    placement.teleport_panel = None;
    placement.selection.clear();
}
