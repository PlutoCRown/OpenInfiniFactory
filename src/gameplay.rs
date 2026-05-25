use bevy::prelude::*;

use crate::blocks::{BlockData, BlockKind};
use crate::inventory::{self, InventoryItems, HOTBAR_SLOTS};
use crate::player::{player_intersects_block, FlyCamera};
use crate::rendering::{despawn_world, rebuild_world, BlockEntity, HoverMarker};
use crate::save::{load_world, next_world_name, save_world, SaveState};
use crate::simulation::reset_simulation;
use crate::state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use crate::world::{grid_to_world, raycast_blocks, seed_demo_world, WorldBlocks};

pub fn gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    simulation: Res<SimulationState>,
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        *mode = match *mode {
            GameMode::Playing | GameMode::Inventory => GameMode::Paused,
            GameMode::Paused => GameMode::Playing,
            GameMode::SaveListPause => GameMode::Paused,
            GameMode::SaveListMain => GameMode::MainMenu,
            other => other,
        };
    }

    if !matches!(*mode, GameMode::Playing | GameMode::Inventory) {
        return;
    }

    if keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::KeyI) {
        *mode = if *mode == GameMode::Inventory {
            GameMode::Playing
        } else {
            GameMode::Inventory
        };
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ] {
        if keys.just_pressed(key) && index < HOTBAR_SLOTS {
            placement.selected = index;
        }
    }

    if keys.just_pressed(KeyCode::KeyR) && !simulation.is_active() {
        placement.facing = placement.facing.rotate();
    }
}

pub fn placement_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    mode: Res<GameMode>,
    simulation: Res<SimulationState>,
    placement: Res<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    player: Query<&Transform, With<FlyCamera>>,
) {
    if *mode != GameMode::Playing {
        return;
    }

    if simulation.is_active() {
        return;
    }

    let Some(target) = placement.target else {
        return;
    };

    let mut changed = false;
    if mouse_buttons.just_pressed(MouseButton::Left) {
        world.blocks.remove(&target.pos);
        changed = true;
    }

    if mouse_buttons.just_pressed(MouseButton::Right) {
        let place_at = target.pos + target.normal;
        if place_at.y >= 0 && world.can_place_solid_at(place_at) {
            if let Ok(player_transform) = player.get_single() {
                if player_intersects_block(player_transform.translation, place_at) {
                    return;
                }
            }

            let Some(kind) = inventory.hotbar[placement.selected] else {
                return;
            };

            if !can_place_in_mode(kind, *builder_mode) {
                return;
            }

            world.blocks.insert(
                place_at,
                BlockData {
                    kind,
                    facing: placement.facing,
                },
            );
            changed = true;
        }
    }

    if changed {
        despawn_world(&mut commands, &block_entities);
        rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
    }
}

pub fn main_menu_actions(
    mut exit: EventWriter<AppExit>,
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<inventory::CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut interactions: Query<
        (&Interaction, &inventory::MainMenuAction),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if *mode != GameMode::MainMenu {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            inventory::MainMenuAction::NewWorld => {
                let name = next_world_name(&save_state.slots);
                world.blocks.clear();
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
                rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
                *mode = GameMode::Playing;
            }
            inventory::MainMenuAction::OpenSaveList => {
                save_state.refresh();
                *mode = GameMode::SaveListMain;
            }
            inventory::MainMenuAction::Quit => {
                exit.send(AppExit::Success);
            }
        }
    }
}

pub fn save_list_actions(
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<inventory::CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut simulation: ResMut<SimulationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut interactions: Query<
        (&Interaction, &inventory::SaveListAction),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !matches!(*mode, GameMode::SaveListMain | GameMode::SaveListPause) {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            inventory::SaveListAction::Load(index) => {
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
                    rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
                    *mode = GameMode::Playing;
                }
            }
            inventory::SaveListAction::Back => {
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
    mut settings: ResMut<GameSettings>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<inventory::CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut world: ResMut<WorldBlocks>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut interactions: Query<
        (&Interaction, &inventory::PauseAction),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if *mode != GameMode::Paused {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            inventory::PauseAction::Resume => *mode = GameMode::Playing,
            inventory::PauseAction::ToggleBuilderMode => {
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
            inventory::PauseAction::SaveWorld => {
                let name = save_state
                    .current
                    .clone()
                    .unwrap_or_else(|| next_world_name(&save_state.slots));
                if save_world(&world, &name) {
                    save_state.current = Some(name);
                    save_state.refresh();
                }
            }
            inventory::PauseAction::OpenSaveList => {
                save_state.refresh();
                *mode = GameMode::SaveListPause;
            }
            inventory::PauseAction::BackToMainMenu => {
                simulation.running = false;
                simulation.accumulator = 0.0;
                world.blocks.clear();
                save_state.current = None;
                despawn_world(&mut commands, &block_entities);
                rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
                *mode = GameMode::MainMenu;
            }
            inventory::PauseAction::FovDown => {
                settings.fov_degrees = (settings.fov_degrees - 5.0).clamp(50.0, 110.0);
            }
            inventory::PauseAction::FovUp => {
                settings.fov_degrees = (settings.fov_degrees + 5.0).clamp(50.0, 110.0);
            }
            inventory::PauseAction::Quit => {
                exit.send(AppExit::Success);
            }
        }
    }
}

pub fn simulation_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut interactions: Query<
        (&Interaction, &inventory::SimulationAction),
        (Changed<Interaction>, With<Button>),
    >,
    builder_mode: Res<BuilderMode>,
    mode: Res<GameMode>,
    mut simulation: ResMut<SimulationState>,
    mut world: ResMut<WorldBlocks>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *builder_mode != BuilderMode::Play || *mode != GameMode::Playing {
        return;
    }

    if keys.just_pressed(KeyCode::KeyF) {
        simulation.running = true;
    }
    simulation.speed = if simulation.running && keys.pressed(KeyCode::KeyF) {
        4.0
    } else {
        1.0
    };

    if keys.just_pressed(KeyCode::KeyR) && simulation.is_active() {
        rollback_simulation(&mut simulation, &mut world);
        despawn_world(&mut commands, &block_entities);
        rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            inventory::SimulationAction::ToggleRun => {
                simulation.running = !simulation.running;
            }
            inventory::SimulationAction::Rollback => {
                rollback_simulation(&mut simulation, &mut world);
                despawn_world(&mut commands, &block_entities);
                rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
            }
        }
    }
}

pub fn apply_fov(
    settings: Res<GameSettings>,
    mut cameras: Query<&mut Projection, With<FlyCamera>>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut projection in &mut cameras {
        if let Projection::Perspective(perspective) = projection.as_mut() {
            perspective.fov = settings.fov_degrees.to_radians();
        }
    }
}

pub fn update_hover(
    mut placement: ResMut<PlacementState>,
    mode: Res<GameMode>,
    camera: Query<&Transform, (With<FlyCamera>, Without<HoverMarker>)>,
    world: Res<WorldBlocks>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<HoverMarker>>,
) {
    if *mode != GameMode::Playing {
        placement.target = None;
        if let Ok((_, mut visibility)) = marker.get_single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Ok(camera_transform) = camera.get_single() else {
        return;
    };

    placement.target = raycast_blocks(
        camera_transform.translation,
        *camera_transform.forward(),
        &world,
    );

    let Ok((mut marker_transform, mut visibility)) = marker.get_single_mut() else {
        return;
    };

    if let Some(target) = placement.target {
        marker_transform.translation = grid_to_world(target.pos);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn can_place_in_mode(kind: BlockKind, mode: BuilderMode) -> bool {
    match mode {
        BuilderMode::Edit => kind.is_scene(),
        BuilderMode::Play => kind.is_factory(),
    }
}

fn rollback_simulation(simulation: &mut SimulationState, world: &mut WorldBlocks) {
    simulation.running = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    reset_simulation(world);
}

fn reset_builder_state(
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut inventory::CarriedItem,
    placement: &mut PlacementState,
) {
    *builder_mode = BuilderMode::Edit;
    *inventory = InventoryItems::for_mode(*builder_mode);
    carried.clear();
    placement.selected = 0;
}
