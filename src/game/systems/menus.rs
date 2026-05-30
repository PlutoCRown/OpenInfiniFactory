use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::ui_widgets::SliderValue;
use bevy::window::{PrimaryWindow, Window};

use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, SolutionState,
    TeleportRenameState, WorldEntryMode,
};
use crate::game::ui::{
    ActiveSettingsSlider, CarriedItem, ConverterAction, GeneratorAction, InventoryItems,
    LabelerAction, MainMenuAction, OpenSettingsDropdown, PauseAction, PendingKeyBind,
    PendingAppExit, SaveListAction, SettingsAction, SettingsSlider, SettingsTab, TeleportAction,
    UiPanelContext, UiPanelId, UiPanelResult, UiRuntime,
};
use crate::game::world::blocks::{MaterialKind, StampColor};
use crate::game::world::grid::{seed_demo_world, WorldBlocks};
use crate::game::world::rendering::{despawn_world, rebuild_world, BlockEntity, WorldRenderAssets};
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{input_from_buttons, open_config_folder, save_config, GameConfig};
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::{
    delete_save, load_world, next_world_name, reset_solution_world, save_solution_with_puzzle,
    save_world, SaveKind, SaveState,
};

pub fn main_menu_actions(
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut pending_exit: ResMut<PendingAppExit>,
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
            MainMenuAction::EditPuzzle => {
                save_state.refresh();
                save_state.selected_puzzle = None;
                save_state.pending_delete = None;
                solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
                *mode = GameMode::SaveListMain;
            }
            MainMenuAction::Play => {
                save_state.refresh();
                save_state.selected_puzzle = None;
                save_state.pending_delete = None;
                solution_state.save_list_entry = WorldEntryMode::PlaySolution;
                *mode = GameMode::SaveListMain;
            }
            MainMenuAction::OpenSettings => {
                ui_runtime.open(
                    UiPanelId::Settings,
                    UiPanelContext::ReturnTo(GameMode::MainMenu),
                );
            }
            MainMenuAction::Quit => {
                request_app_exit(&mut pending_exit, AppExit::Success);
            }
        }
    }
}

pub fn app_exit_requests(
    mut commands: Commands,
    mut app_exit_messages: ResMut<Messages<AppExit>>,
    mut pending_exit: ResMut<PendingAppExit>,
    primary_windows: Query<Entity, (With<Window>, With<PrimaryWindow>)>,
    windows: Query<Entity, With<Window>>,
) {
    let mut drained_exit = None;
    for exit in app_exit_messages.drain() {
        if exit.is_error() {
            drained_exit = Some(exit);
            break;
        }
        drained_exit.get_or_insert(exit);
    }

    if let Some(requested_exit) = drained_exit {
        pending_exit.requested = true;
        pending_exit.exit = Some(requested_exit);
    }

    if !pending_exit.requested {
        return;
    }

    if windows.is_empty() {
        app_exit_messages.write(pending_exit.exit.take().unwrap_or(AppExit::Success));
        pending_exit.requested = false;
        return;
    }

    if let Ok(entity) = primary_windows.single() {
        commands.entity(entity).despawn();
    } else {
        for entity in &windows {
            commands.entity(entity).despawn();
        }
    }
}

fn request_app_exit(pending_exit: &mut PendingAppExit, exit: AppExit) {
    pending_exit.requested = true;
    pending_exit.exit = Some(exit);
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
    if *mode != GameMode::SaveListMain {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            SaveListAction::NewPuzzle => {
                if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                    continue;
                }
                let name = next_world_name(&save_state.slots);
                world.clear();
                seed_demo_world(&mut world);
                save_world(&world, &name, SaveKind::Puzzle);
                save_state.refresh();
                open_loaded_world(
                    &name,
                    WorldEntryMode::EditPuzzle,
                    &mut world,
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut commands,
                    &block_entities,
                    &render_assets,
                    &mut mode,
                );
            }
            SaveListAction::NewSolution => {
                if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                    continue;
                }
                let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                    continue;
                };
                if load_world(&mut world, &puzzle_name).is_none() {
                    continue;
                }
                let name = next_world_name(&save_state.slots);
                let puzzle_snapshot = world.clone();
                save_solution_with_puzzle(&world, &name, &puzzle_snapshot);
                save_state.refresh();
                open_loaded_world(
                    &name,
                    WorldEntryMode::PlaySolution,
                    &mut world,
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut commands,
                    &block_entities,
                    &render_assets,
                    &mut mode,
                );
            }
            SaveListAction::LoadPuzzle(index) => {
                let puzzles = save_state.puzzles();
                let Some(entry) = puzzles.get(index) else {
                    continue;
                };
                if solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                    let name = entry.name.clone();
                    open_loaded_world(
                        &name,
                        WorldEntryMode::EditPuzzle,
                        &mut world,
                        &mut builder_mode,
                        &mut inventory,
                        &mut carried,
                        &mut placement,
                        &mut save_state,
                        &mut solution_state,
                        &mut simulation,
                        &mut commands,
                        &block_entities,
                        &render_assets,
                        &mut mode,
                    );
                } else {
                    save_state.selected_puzzle = Some(entry.name.clone());
                }
            }
            SaveListAction::LoadSolution(index) => {
                if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                    continue;
                }
                if save_state.selected_puzzle.is_none() {
                    continue;
                }
                let Some(puzzle_name) = save_state.selected_puzzle.as_deref() else {
                    continue;
                };
                let solutions = save_state.solutions_for_puzzle(puzzle_name);
                let Some(entry) = solutions.get(index) else {
                    continue;
                };
                let name = entry.name.clone();
                open_loaded_world(
                    &name,
                    WorldEntryMode::PlaySolution,
                    &mut world,
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut commands,
                    &block_entities,
                    &render_assets,
                    &mut mode,
                );
            }
            SaveListAction::DeletePuzzle(index) => {
                if let Some(entry) = save_state.puzzles().get(index) {
                    save_state.pending_delete = Some(entry.name.clone());
                }
            }
            SaveListAction::DeleteSolution(index) => {
                let Some(puzzle_name) = save_state.selected_puzzle.as_deref() else {
                    continue;
                };
                if let Some(entry) = save_state.solutions_for_puzzle(puzzle_name).get(index) {
                    save_state.pending_delete = Some(entry.name.clone());
                }
            }
            SaveListAction::ConfirmDelete => {
                if let Some(name) = save_state.pending_delete.take() {
                    delete_save(&name);
                    save_state.refresh();
                    if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
                        save_state.selected_puzzle = None;
                    }
                }
            }
            SaveListAction::CancelDelete => {
                save_state.pending_delete = None;
            }
            SaveListAction::Back => {
                save_state.pending_delete = None;
                *mode = GameMode::MainMenu;
            }
        }
    }
}

pub fn pause_menu_actions(
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut world: ResMut<WorldBlocks>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    render_assets: Res<WorldRenderAssets>,
    mut interactions: Query<(&Interaction, &PauseAction), (Changed<Interaction>, With<Button>)>,
) {
    if !matches!(
        *mode,
        GameMode::Paused | GameMode::ConfirmSaveSolutionBeforeEdit | GameMode::ConfirmBackToMain
    ) {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            PauseAction::Resume => *mode = GameMode::Playing,
            PauseAction::ToggleBuilderMode => {
                if solution_state.entry == WorldEntryMode::PlaySolution {
                    continue;
                }
                *builder_mode = match *builder_mode {
                    BuilderMode::Edit => {
                        simulation.running = false;
                        simulation.step_requested = false;
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
                *mode = GameMode::Playing;
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
            PauseAction::SaveAndBackToMain => {
                save_current_world(&world, &mut save_state, &mut solution_state);
                clear_loaded_world(
                    &mut world,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut commands,
                    &block_entities,
                    &render_assets,
                );
                *mode = GameMode::MainMenu;
            }
            PauseAction::DiscardAndBackToMain => {
                clear_loaded_world(
                    &mut world,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut commands,
                    &block_entities,
                    &render_assets,
                );
                *mode = GameMode::MainMenu;
            }
            PauseAction::CancelBackToMain => {
                *mode = GameMode::Paused;
            }
            PauseAction::ResetSolution => {
                if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
                    reset_solution_world(&mut world, puzzle_snapshot);
                    simulation.running = false;
                    simulation.step_requested = false;
                    simulation.turn = 0;
                    simulation.accumulator = 0.0;
                    simulation.start_snapshot = None;
                    despawn_world(&mut commands, &block_entities);
                    rebuild_world(&mut commands, &world, &render_assets);
                }
            }
            PauseAction::OpenSettings => {
                ui_runtime.open(
                    UiPanelId::Settings,
                    UiPanelContext::ReturnTo(GameMode::Paused),
                );
            }
            PauseAction::BackToMainMenu => {
                if solution_state.dirty {
                    *mode = GameMode::ConfirmBackToMain;
                } else {
                    clear_loaded_world(
                        &mut world,
                        &mut placement,
                        &mut save_state,
                        &mut solution_state,
                        &mut simulation,
                        &mut commands,
                        &block_entities,
                        &render_assets,
                    );
                    *mode = GameMode::MainMenu;
                }
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
        solution_state.dirty = false;
        save_state.refresh();
    }
}

fn open_loaded_world(
    name: &str,
    entry: WorldEntryMode,
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    mode: &mut GameMode,
) {
    let Some(loaded) = load_world(world, name) else {
        return;
    };

    simulation.running = false;
    simulation.step_requested = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    carried.clear();

    *builder_mode = match entry {
        WorldEntryMode::EditPuzzle => BuilderMode::Edit,
        WorldEntryMode::PlaySolution => BuilderMode::Play,
    };
    *inventory = InventoryItems::for_mode(*builder_mode);
    placement.selected = 0;

    save_state.current = Some(name.to_string());
    save_state.current_kind = Some(match entry {
        WorldEntryMode::EditPuzzle => SaveKind::Puzzle,
        WorldEntryMode::PlaySolution => SaveKind::Solution,
    });
    save_state.selected_puzzle = None;
    save_state.pending_delete = None;

    solution_state.entry = entry;
    solution_state.dirty = false;
    solution_state.puzzle_snapshot = match entry {
        WorldEntryMode::EditPuzzle => None,
        WorldEntryMode::PlaySolution => loaded.puzzle_snapshot.or_else(|| Some(loaded.world)),
    };

    despawn_world(commands, block_entities);
    rebuild_world(commands, world, render_assets);
    *mode = GameMode::Playing;
}

fn clear_loaded_world(
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
) {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    world.clear();
    save_state.current = None;
    save_state.current_kind = None;
    save_state.selected_puzzle = None;
    save_state.pending_delete = None;
    solution_state.puzzle_snapshot = None;
    solution_state.dirty = false;
    solution_state.entry = WorldEntryMode::EditPuzzle;
    despawn_world(commands, block_entities);
    rebuild_world(commands, world, render_assets);
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
    save_state.current_kind = Some(SaveKind::Puzzle);
    solution_state.puzzle_snapshot = None;
    *mode = GameMode::Paused;
}

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mode: ResMut<GameMode>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut i18n: ResMut<I18n>,
    mut settings_tab: ResMut<SettingsTab>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut interactions: Query<(Ref<Interaction>, &SettingsAction, Option<&SliderValue>), With<Button>>,
) {
    if !ui_runtime.is_settings_open() {
        pending_key_bind.0 = None;
        open_dropdown.0 = None;
        active_slider.0 = None;
        return;
    }

    if let Some(action) = pending_key_bind.0 {
        if let Some(input) = input_from_buttons(&keys, &mouse_buttons) {
            config.set_input(action, input);
            save_config(&config);
            pending_key_bind.0 = None;
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(slider) = active_slider.0.take() {
            if let Some((_, _, value)) = interactions.iter().find(|(_, action, _)| {
                matches!(
                    (*action, slider),
                    (SettingsAction::FovSlider, SettingsSlider::Fov)
                        | (SettingsAction::UiScaleSlider, SettingsSlider::UiScale)
                        | (SettingsAction::GravitySlider, SettingsSlider::Gravity)
                )
            }) {
                if let Some(value) = value {
                    apply_settings_slider(
                        slider,
                        value.0 / 100.0,
                        &mut settings,
                        &mut ui_scale,
                        &mut config,
                    );
                    save_config(&config);
                }
            }
        }
    }

    for (interaction, action, _) in &mut interactions {
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
                active_slider.0 = Some(SettingsSlider::Fov);
            }
            SettingsAction::UiScaleSlider => {
                active_slider.0 = Some(SettingsSlider::UiScale);
            }
            SettingsAction::GravitySlider => {
                active_slider.0 = Some(SettingsSlider::Gravity);
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
                settings.gravity_scale = config
                    .gravity_scale
                    .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
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
                let return_mode = settings_return_mode(&ui_runtime, *mode);
                ui_runtime.close_active(UiPanelResult::SettingsClosed);
                *mode = return_mode;
            }
        }
    }
}

fn settings_return_mode(ui_runtime: &UiRuntime, fallback: GameMode) -> GameMode {
    ui_runtime
        .active()
        .and_then(|session| match session.context {
            UiPanelContext::ReturnTo(mode) => Some(mode),
            _ => None,
        })
        .unwrap_or(fallback)
}

fn apply_settings_slider(
    slider: SettingsSlider,
    percent: f32,
    settings: &mut GameSettings,
    ui_scale: &mut UiScale,
    config: &mut GameConfig,
) {
    match slider {
        SettingsSlider::Fov => {
            settings.fov_degrees = (50.0 + percent * 60.0).round().clamp(50.0, 110.0);
            config.fov_degrees = settings.fov_degrees;
        }
        SettingsSlider::UiScale => {
            settings.ui_scale =
                ((UI_SCALE_MIN + percent * (UI_SCALE_MAX - UI_SCALE_MIN)) * 10.0).round() / 10.0;
            settings.ui_scale = settings.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
            ui_scale.0 = settings.ui_scale;
            config.ui_scale = settings.ui_scale;
        }
        SettingsSlider::Gravity => {
            settings.gravity_scale =
                ((GRAVITY_SCALE_MIN + percent * (GRAVITY_SCALE_MAX - GRAVITY_SCALE_MIN)) * 10.0)
                    .round()
                    / 10.0;
            settings.gravity_scale = settings
                .gravity_scale
                .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
            config.gravity_scale = settings.gravity_scale;
        }
    }
}

pub fn generator_menu_actions(
    mut ui_runtime: ResMut<UiRuntime>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut interactions: Query<(&Interaction, &GeneratorAction), (Changed<Interaction>, With<Button>)>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Generator) {
        return;
    }

    let Some(pos) = ui_runtime.active_block_pos() else {
        ui_runtime.close_current();
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
                solution_state.dirty = true;
            }
            GeneratorAction::PeriodUp => {
                settings.period = (settings.period + 1).min(120);
                world.set_generator_settings(pos, settings);
                solution_state.dirty = true;
            }
            GeneratorAction::MaterialNext => {
                settings.material = next_material(settings.material);
                world.set_generator_settings(pos, settings);
                solution_state.dirty = true;
            }
            GeneratorAction::Close => {
                ui_runtime.close_active(UiPanelResult::BlockClosed { pos });
            }
        }
    }
}

pub fn labeler_menu_actions(
    mut ui_runtime: ResMut<UiRuntime>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut interactions: Query<(&Interaction, &LabelerAction), (Changed<Interaction>, With<Button>)>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Labeler) {
        return;
    }

    let Some(pos) = ui_runtime.active_block_pos() else {
        ui_runtime.close_current();
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
                solution_state.dirty = true;
            }
            LabelerAction::NextColor => {
                settings.color = next_stamp_color(settings.color);
                world.set_labeler_settings(pos, settings);
                solution_state.dirty = true;
            }
            LabelerAction::Close => {
                ui_runtime.close_active(UiPanelResult::BlockClosed { pos });
            }
        }
    }
}

pub fn converter_menu_actions(
    mut ui_runtime: ResMut<UiRuntime>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut interactions: Query<(&Interaction, &ConverterAction), (Changed<Interaction>, With<Button>)>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Converter) {
        return;
    }

    let Some(pos) = ui_runtime.active_block_pos() else {
        ui_runtime.close_current();
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
                solution_state.dirty = true;
            }
            ConverterAction::InputNext => {
                settings.input = next_material(settings.input);
                world.set_converter_settings(pos, settings);
                solution_state.dirty = true;
            }
            ConverterAction::OutputNext => {
                settings.output = next_material(settings.output);
                world.set_converter_settings(pos, settings);
                solution_state.dirty = true;
            }
            ConverterAction::Close => {
                ui_runtime.close_active(UiPanelResult::BlockClosed { pos });
            }
        }
    }
}

pub fn teleport_menu_actions(
    mut ui_runtime: ResMut<UiRuntime>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut interactions: Query<(&Interaction, &TeleportAction), (Changed<Interaction>, With<Button>)>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Teleport) {
        return;
    }

    let Some(pos) = ui_runtime.active_block_pos() else {
        ui_runtime.close_current();
        return;
    };

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            TeleportAction::CyclePair => {
                world.cycle_teleport_pair(pos);
                solution_state.dirty = true;
            }
            TeleportAction::Rename => {
                let settings = world.teleport_settings(pos);
                rename_state.editing = Some(pos);
                rename_state.buffer = settings.name;
            }
            TeleportAction::Close => {
                rename_state.editing = None;
                ui_runtime.close_active(UiPanelResult::BlockClosed { pos });
            }
        }
    }
}

pub fn teleport_rename_input(
    ui_runtime: Res<UiRuntime>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut keyboard_input: MessageReader<KeyboardInput>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Teleport) || rename_state.editing.is_none() {
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
            solution_state.dirty = true;
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
