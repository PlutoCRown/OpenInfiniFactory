use bevy::prelude::*;

use crate::blocks::{BlockData, BlockKind};
use crate::inventory::{self, InventoryItems, HOTBAR_SLOTS};
use crate::player::{player_intersects_block, FlyCamera};
use crate::rendering::{despawn_world, rebuild_world, BlockEntity, HoverMarker};
use crate::save::{load_world, save_world};
use crate::state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use crate::world::{grid_to_world, raycast_blocks, WorldBlocks};

pub fn gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
) {
    if keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::KeyI) {
        *mode = if *mode == GameMode::Inventory {
            GameMode::Playing
        } else {
            GameMode::Inventory
        };
    }

    if keys.just_pressed(KeyCode::Escape) {
        *mode = if *mode == GameMode::Playing {
            GameMode::Paused
        } else {
            GameMode::Playing
        };
    }

    if *mode == GameMode::Paused && mouse_buttons.just_pressed(MouseButton::Left) {
        *mode = GameMode::Playing;
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

    if keys.just_pressed(KeyCode::KeyR) {
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
    placement: Res<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    player: Query<&Transform, With<FlyCamera>>,
) {
    if *mode != GameMode::Playing {
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
        if place_at.y >= 0 && !world.blocks.contains_key(&place_at) {
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

pub fn save_load_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
) {
    if keys.just_pressed(KeyCode::F5) {
        save_world(&world);
    }

    if keys.just_pressed(KeyCode::F9) && load_world(&mut world) {
        despawn_world(&mut commands, &block_entities);
        rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
    }
}

pub fn pause_menu_actions(
    mut exit: EventWriter<AppExit>,
    mut settings: ResMut<GameSettings>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut mode: ResMut<GameMode>,
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
    mut interactions: Query<
        (&Interaction, &inventory::SimulationAction),
        (Changed<Interaction>, With<Button>),
    >,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
) {
    if *builder_mode != BuilderMode::Play {
        return;
    }

    for (interaction, action) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match action {
            inventory::SimulationAction::ToggleRun => {
                simulation.running = !simulation.running;
            }
            inventory::SimulationAction::Faster => {
                simulation.speed = match simulation.speed as u32 {
                    1 => 2.0,
                    2 => 4.0,
                    _ => 1.0,
                };
            }
            inventory::SimulationAction::Rollback => {
                simulation.running = false;
                simulation.turn = 0;
                simulation.accumulator = 0.0;
            }
        }
    }
}

pub fn simulation_tick(
    time: Res<Time>,
    builder_mode: Res<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
) {
    if *builder_mode != BuilderMode::Play || !simulation.running {
        return;
    }

    simulation.accumulator += time.delta_seconds() * simulation.speed;
    while simulation.accumulator >= 1.0 {
        simulation.turn += 1;
        simulation.accumulator -= 1.0;
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
        BuilderMode::Edit => matches!(kind, BlockKind::Solid | BlockKind::Glass),
        BuilderMode::Play => matches!(
            kind,
            BlockKind::Conveyor | BlockKind::Piston | BlockKind::Goal
        ),
    }
}
