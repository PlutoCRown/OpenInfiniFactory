use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SettingsReturnMode, SimulationState,
};
use crate::game::ui::{
    CarriedItem, GeneratorAction, InventoryItems, MainMenuAction, OpenSettingsDropdown,
    PauseAction, PendingKeyBind, SaveListAction, SettingsAction, SettingsTab,
};
use crate::game::world::blocks::MaterialKind;
use crate::game::world::grid::{seed_demo_world, WorldBlocks};
use crate::game::world::rendering::{despawn_world, rebuild_world, BlockEntity, WorldRenderAssets};
use crate::game::{UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{key_from_input, open_config_folder, save_config, GameConfig};
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::{load_world, next_world_name, save_world, SaveState};

pub fn main_menu_actions(
    mut exit: EventWriter<AppExit>,
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
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
                save_world(&world, &name);
                save_state.current = Some(name);
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
                if load_world(&mut world, &name) {
                    save_state.current = Some(name);
                    simulation.running = false;
                    simulation.turn = 0;
                    simulation.accumulator = 0.0;
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
    mut settings_return: ResMut<SettingsReturnMode>,
    mut world: ResMut<WorldBlocks>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    render_assets: Res<WorldRenderAssets>,
    mut interactions: Query<(&Interaction, &PauseAction), (Changed<Interaction>, With<Button>)>,
) {
    if *mode != GameMode::Paused {
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
                        BuilderMode::Play
                    }
                    BuilderMode::Play => BuilderMode::Edit,
                };
                *inventory = InventoryItems::for_mode(*builder_mode);
                carried.clear();
                placement.selected = 0;
            }
            PauseAction::SaveWorld => {
                let name = save_state
                    .current
                    .clone()
                    .unwrap_or_else(|| next_world_name(&save_state.slots));
                if save_world(&world, &name) {
                    save_state.current = Some(name);
                    save_state.refresh();
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
                placement.generator_panel = None;
                world.clear();
                save_state.current = None;
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

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
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
        if let Some(key) = key_from_input(&keys) {
            config.set_key(action, key);
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

fn next_material(material: MaterialKind) -> MaterialKind {
    let all = MaterialKind::ALL;
    let index = all
        .iter()
        .position(|candidate| *candidate == material)
        .unwrap_or(0);
    all[(index + 1) % all.len()]
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
    placement.selection.clear();
}
