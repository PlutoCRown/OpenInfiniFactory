use bevy::prelude::*;

use crate::game::player::controller::{player_intersects_block, FlyCamera};
use crate::game::state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use crate::game::ui::{InventoryItems, PendingKeyBind, HOTBAR_SLOTS};
use crate::game::world::blocks::{BlockData, BlockKind};
use crate::game::world::grid::{grid_to_world, raycast_blocks, WorldBlocks};
use crate::game::world::rendering::{
    spawn_block, BlockEntity, HoverMarker, PlacementPreview, WorldRenderAssets,
};
use crate::shared::config::GameConfig;

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
    builder_mode: Res<BuilderMode>,
    mode: Res<GameMode>,
    simulation: Res<SimulationState>,
    mut placement: ResMut<PlacementState>,
    render_assets: Res<WorldRenderAssets>,
    block_entities: Query<(Entity, &BlockEntity)>,
    player: Query<&Transform, With<FlyCamera>>,
) {
    let place_button = MouseButton::Left;
    let delete_button = MouseButton::Right;

    if *mode != GameMode::Playing {
        placement.pending_delete = None;
        return;
    }

    if simulation.is_active() {
        placement.pending_delete = None;
        return;
    }

    let mut canceled_delete = false;

    if mouse_buttons.just_pressed(place_button) && placement.pending_delete.is_some() {
        placement.pending_delete = None;
        canceled_delete = true;
    }

    if mouse_buttons.just_released(delete_button) {
        if let Some(delete_pos) = placement.pending_delete.take() {
            if world.blocks.remove(&delete_pos).is_some() {
                if let Some((entity, _)) = block_entities
                    .iter()
                    .find(|(_, block_entity)| block_entity.pos == delete_pos)
                {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }

    if let Some(target) = placement.target {
        if mouse_buttons.just_pressed(delete_button) {
            placement.pending_delete = Some(target.pos);
        }

        if mouse_buttons.just_pressed(place_button)
            && placement.pending_delete.is_none()
            && !canceled_delete
        {
            let place_at = target.pos + target.normal;
            if can_place_selected_block(
                place_at,
                &inventory,
                *builder_mode,
                &placement,
                &world,
                &player,
            ) {
                let kind = inventory.hotbar[placement.selected].expect("validated selected block");
                world.blocks.insert(
                    place_at,
                    BlockData {
                        kind,
                        facing: placement.facing,
                    },
                );
                spawn_block(
                    &mut commands,
                    &render_assets,
                    &world,
                    place_at,
                    BlockData {
                        kind,
                        facing: placement.facing,
                    },
                );
            }
        }
    }
}

fn can_place_selected_block(
    place_at: IVec3,
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    placement: &PlacementState,
    world: &WorldBlocks,
    player: &Query<&Transform, With<FlyCamera>>,
) -> bool {
    if place_at.y < 0 || !world.can_place_solid_at(place_at) {
        return false;
    }

    let Some(kind) = inventory.hotbar[placement.selected] else {
        return false;
    };

    if !can_place_in_mode(kind, builder_mode) {
        return false;
    }

    if let Ok(player_transform) = player.get_single() {
        if player_intersects_block(player_transform.translation, place_at) {
            return false;
        }
    }

    true
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
        placement.pending_delete = None;
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

    if let Some(delete_pos) = placement.pending_delete {
        marker_transform.translation = grid_to_world(delete_pos);
        *marker_visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(marker_material) {
            material.base_color = Color::srgba(1.0, 0.12, 0.08, 0.34);
        }
    } else if let Some(target) = placement.target {
        marker_transform.translation = grid_to_world(target.pos);
        *marker_visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(marker_material) {
            material.base_color = Color::srgba(1.0, 1.0, 1.0, 0.16);
        }
    } else {
        *marker_visibility = Visibility::Hidden;
    }

    let Ok((mut preview_transform, mut preview_visibility, preview_material)) =
        preview.get_single_mut()
    else {
        return;
    };

    if placement.pending_delete.is_some() {
        *preview_visibility = Visibility::Hidden;
        return;
    }

    let Some(target) = placement.target else {
        *preview_visibility = Visibility::Hidden;
        return;
    };

    let place_at = target.pos + target.normal;
    if can_place_selected_block(
        place_at,
        &inventory,
        *builder_mode,
        &placement,
        &world,
        &player,
    ) {
        preview_transform.translation = grid_to_world(place_at);
        *preview_visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(preview_material) {
            let kind = inventory.hotbar[placement.selected].expect("validated selected block");
            material.base_color = translucent_color(kind.material(), 0.34);
            material.unlit = kind == BlockKind::WeldPoint;
        }
    } else {
        *preview_visibility = Visibility::Hidden;
    }
}

fn translucent_color(color: Color, alpha: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(srgba.red, srgba.green, srgba.blue, alpha)
}

fn can_place_in_mode(kind: BlockKind, mode: BuilderMode) -> bool {
    match mode {
        BuilderMode::Edit => kind.is_scene(),
        BuilderMode::Play => kind.is_factory(),
    }
}
