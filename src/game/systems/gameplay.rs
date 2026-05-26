use bevy::prelude::*;

use crate::game::player::controller::{player_intersects_block, FlyCamera};
use crate::game::state::{
    BuilderMode, EditGesture, EditGestureKind, GameMode, GameSettings, PlacementState,
    SimulationState,
};
use crate::game::ui::{InventoryItems, PendingKeyBind, HOTBAR_SLOTS};
use crate::game::world::blocks::{BlockData, BlockKind};
use crate::game::world::grid::{grid_to_world, raycast_blocks, WorldBlocks};
use crate::game::world::rendering::{
    despawn_edit_previews, spawn_block, spawn_edit_preview, BlockEntity, EditPreview,
    EditPreviewKind, HoverMarker, PlacementPreview, WorldRenderAssets,
};
use crate::shared::config::{ConfigSelectionMode, GameConfig};

pub fn gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    simulation: Res<SimulationState>,
    pending_key_bind: Res<PendingKeyBind>,
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
) {
    let bindings = &config.key_bindings;

    if *mode == GameMode::Settings && pending_key_bind.0.is_some() {
        return;
    }

    if keys.just_pressed(bindings.pause.key_code()) {
        *mode = match *mode {
            GameMode::Playing | GameMode::Inventory => GameMode::Paused,
            GameMode::Paused => GameMode::Playing,
            GameMode::Settings => GameMode::Paused,
            GameMode::SaveListPause => GameMode::Paused,
            GameMode::SaveListMain => GameMode::MainMenu,
            other => other,
        };
    }

    if !matches!(*mode, GameMode::Playing | GameMode::Inventory) {
        return;
    }

    if keys.just_pressed(bindings.inventory.key_code()) {
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

    if keys.just_pressed(bindings.rotate_or_rollback.key_code()) && !simulation.is_active() {
        placement.facing = placement.facing.rotate();
    }
}

pub fn placement_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    inventory: Res<InventoryItems>,
    config: Res<GameConfig>,
    builder_mode: Res<BuilderMode>,
    mode: Res<GameMode>,
    simulation: Res<SimulationState>,
    mut placement: ResMut<PlacementState>,
    render_assets: Res<WorldRenderAssets>,
    block_entities: Query<(Entity, &BlockEntity)>,
    edit_previews: Query<Entity, With<EditPreview>>,
    player: Query<&Transform, With<FlyCamera>>,
) {
    let place_button = MouseButton::Left;
    let delete_button = MouseButton::Right;

    if *mode != GameMode::Playing {
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if simulation.is_active() {
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    let current_place_at = placement.target.map(|target| target.pos + target.normal);
    let current_delete_at = placement.target.map(|target| target.pos);

    if mouse_buttons.just_pressed(delete_button) {
        match placement.edit_gesture.as_mut() {
            Some(gesture) if matches!(gesture.kind, EditGestureKind::Place { .. }) => {
                gesture.canceled = true;
            }
            None => {
                if let Some(start) = current_delete_at {
                    placement.edit_gesture = Some(EditGesture {
                        kind: EditGestureKind::Delete,
                        start,
                        canceled: false,
                    });
                }
            }
            Some(_) => {}
        }
    }

    if mouse_buttons.just_pressed(place_button) {
        match placement.edit_gesture.as_mut() {
            Some(gesture) if matches!(gesture.kind, EditGestureKind::Delete) => {
                gesture.canceled = true;
            }
            None => {
                if let Some(start) = current_place_at {
                    if let Some(block) = selected_place_block(&inventory, *builder_mode, &placement)
                    {
                        placement.edit_gesture = Some(EditGesture {
                            kind: EditGestureKind::Place { block },
                            start,
                            canceled: false,
                        });
                    }
                }
            }
            Some(_) => {}
        }
    }

    let released_place = mouse_buttons.just_released(place_button);
    let released_delete = mouse_buttons.just_released(delete_button);
    let should_finish = placement.edit_gesture.as_ref().is_some_and(|gesture| {
        matches!(gesture.kind, EditGestureKind::Place { .. }) && released_place
            || matches!(gesture.kind, EditGestureKind::Delete) && released_delete
    });

    if should_finish {
        if let Some(gesture) = placement.edit_gesture.take() {
            if !gesture.canceled {
                commit_edit_gesture(
                    gesture,
                    current_place_at,
                    current_delete_at,
                    &config,
                    &mut world,
                    *builder_mode,
                    &player,
                    &mut commands,
                    &render_assets,
                    &block_entities,
                );
            }
        }
    }

    despawn_edit_previews(&mut commands, &edit_previews);
    if let Some(gesture) = &placement.edit_gesture {
        if !gesture.canceled {
            spawn_gesture_previews(
                gesture,
                current_place_at,
                current_delete_at,
                &config,
                &world,
                *builder_mode,
                &player,
                &mut commands,
                &render_assets,
            );
        }
    }
}

fn selected_place_block(
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    placement: &PlacementState,
) -> Option<BlockData> {
    let kind = inventory.hotbar[placement.selected]?;
    can_place_in_mode(kind, builder_mode).then_some(BlockData {
        kind,
        facing: placement.facing,
    })
}

fn can_place_block_at(
    place_at: IVec3,
    block: BlockData,
    builder_mode: BuilderMode,
    world: &WorldBlocks,
    player: &Query<&Transform, With<FlyCamera>>,
) -> bool {
    if place_at.y < 0 || !world.can_place_solid_at(place_at) {
        return false;
    }

    if !can_place_in_mode(block.kind, builder_mode) {
        return false;
    }

    if let Ok(player_transform) = player.get_single() {
        if player_intersects_block(player_transform.translation, place_at) {
            return false;
        }
    }

    true
}

fn commit_edit_gesture(
    gesture: EditGesture,
    current_place_at: Option<IVec3>,
    current_delete_at: Option<IVec3>,
    config: &GameConfig,
    world: &mut WorldBlocks,
    builder_mode: BuilderMode,
    player: &Query<&Transform, With<FlyCamera>>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
    block_entities: &Query<(Entity, &BlockEntity)>,
) {
    match gesture.kind {
        EditGestureKind::Place { block } => {
            let positions = selection_positions(
                config.place_selection_mode,
                gesture.start,
                current_place_at.unwrap_or(gesture.start),
            );
            for pos in positions {
                if can_place_block_at(pos, block, builder_mode, world, player) {
                    world.insert(pos, block);
                    spawn_block(commands, render_assets, world, pos, block);
                }
            }
        }
        EditGestureKind::Delete => {
            let positions = selection_positions(
                config.delete_selection_mode,
                gesture.start,
                current_delete_at.unwrap_or(gesture.start),
            );
            for pos in positions {
                if world.remove(&pos).is_some() {
                    if let Some((entity, _)) = block_entities
                        .iter()
                        .find(|(_, block_entity)| block_entity.pos == pos)
                    {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn spawn_gesture_previews(
    gesture: &EditGesture,
    current_place_at: Option<IVec3>,
    current_delete_at: Option<IVec3>,
    config: &GameConfig,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    player: &Query<&Transform, With<FlyCamera>>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) {
    match gesture.kind {
        EditGestureKind::Place { block } => {
            let positions = selection_positions(
                config.place_selection_mode,
                gesture.start,
                current_place_at.unwrap_or(gesture.start),
            );
            for pos in positions {
                if can_place_block_at(pos, block, builder_mode, world, player) {
                    spawn_edit_preview(commands, render_assets, pos, EditPreviewKind::Place);
                }
            }
        }
        EditGestureKind::Delete => {
            let positions = selection_positions(
                config.delete_selection_mode,
                gesture.start,
                current_delete_at.unwrap_or(gesture.start),
            );
            for pos in positions {
                if world.is_occupied(pos) {
                    spawn_edit_preview(commands, render_assets, pos, EditPreviewKind::Delete);
                }
            }
        }
    }
}

fn selection_positions(mode: ConfigSelectionMode, start: IVec3, end: IVec3) -> Vec<IVec3> {
    match mode {
        ConfigSelectionMode::Point => vec![start],
        ConfigSelectionMode::Line => line_selection(start, end),
        ConfigSelectionMode::Plane => plane_selection(start, end),
    }
}

fn line_selection(start: IVec3, end: IVec3) -> Vec<IVec3> {
    let delta = end - start;
    let axis = if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        0
    } else if delta.y.abs() >= delta.z.abs() {
        1
    } else {
        2
    };

    let (min, max) = match axis {
        0 => min_max(start.x, end.x),
        1 => min_max(start.y, end.y),
        _ => min_max(start.z, end.z),
    };

    (min..=max)
        .map(|value| match axis {
            0 => IVec3::new(value, start.y, start.z),
            1 => IVec3::new(start.x, value, start.z),
            _ => IVec3::new(start.x, start.y, value),
        })
        .collect()
}

fn plane_selection(start: IVec3, end: IVec3) -> Vec<IVec3> {
    let (min_x, max_x) = min_max(start.x, end.x);
    let (min_z, max_z) = min_max(start.z, end.z);
    let mut positions = Vec::new();
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            positions.push(IVec3::new(x, start.y, z));
        }
    }
    positions
}

fn min_max(a: i32, b: i32) -> (i32, i32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
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
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    camera: Query<&Transform, (With<FlyCamera>, Without<HoverMarker>)>,
    world: Res<WorldBlocks>,
    player: Query<&Transform, With<FlyCamera>>,
    mut marker: Query<
        (&mut Transform, &mut Visibility, &Handle<StandardMaterial>),
        (With<HoverMarker>, Without<FlyCamera>),
    >,
    mut preview: Query<
        (&mut Transform, &mut Visibility, &Handle<StandardMaterial>),
        (
            With<PlacementPreview>,
            Without<HoverMarker>,
            Without<FlyCamera>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *mode != GameMode::Playing {
        placement.target = None;
        placement.edit_gesture = None;
        if let Ok((_, mut visibility, _)) = marker.get_single_mut() {
            *visibility = Visibility::Hidden;
        }
        if let Ok((_, mut visibility, _)) = preview.get_single_mut() {
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

    let Ok((mut marker_transform, mut marker_visibility, marker_material)) =
        marker.get_single_mut()
    else {
        return;
    };

    if placement.edit_gesture.is_some() {
        *marker_visibility = Visibility::Hidden;
    } else if let Some(target) = placement.target {
        marker_transform.translation = grid_to_world(target.pos);
        *marker_visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(marker_material) {
            material.base_color = Color::srgba(1.0, 1.0, 1.0, 0.16);
        }
    } else {
        *marker_visibility = Visibility::Hidden;
    }

    let Ok((_, mut preview_visibility, _)) = preview.get_single_mut() else {
        return;
    };

    let _ = (inventory, builder_mode, player);
    *preview_visibility = Visibility::Hidden;
}

fn can_place_in_mode(kind: BlockKind, mode: BuilderMode) -> bool {
    match mode {
        BuilderMode::Edit => kind.is_scene(),
        BuilderMode::Play => kind.is_factory(),
    }
}
