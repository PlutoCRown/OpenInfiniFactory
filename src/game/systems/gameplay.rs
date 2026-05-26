use bevy::prelude::*;

use crate::game::player::controller::{player_intersects_block, FlyCamera};
use crate::game::state::{
    BuilderMode, EditGesture, EditGestureKind, GameMode, GameSettings, PlacementState,
    SelectionAxis, SelectionBounds, SelectionDrag, SimulationState,
};
use crate::game::ui::{AreaKind, CarriedItem, InventoryItems, PendingKeyBind, HOTBAR_SLOTS};
use crate::game::world::animation::BlockAnimation;
use crate::game::world::blocks::{BlockData, BlockKind};
use crate::game::world::grid::{grid_to_world, raycast_blocks, MaterialWeld, WorldBlocks};
use crate::game::world::rendering::{
    despawn_edit_previews, rebuild_world, rebuild_world_with_animations, spawn_block_preview,
    spawn_block_with_animation, spawn_edit_preview, BlockEntity, EditPreview, EditPreviewKind,
    HoverMarker, PlacementPreview, WorldRenderAssets,
};
use crate::shared::config::{ConfigSelectionMode, GameConfig};

pub fn gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    simulation: Res<SimulationState>,
    pending_key_bind: Res<PendingKeyBind>,
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
    mut carried: ResMut<CarriedItem>,
) {
    let bindings = &config.key_bindings;

    if *mode == GameMode::Settings && pending_key_bind.0.is_some() {
        return;
    }

    if keys.just_pressed(bindings.pause.key_code()) {
        *mode = match *mode {
            GameMode::Playing => GameMode::Paused,
            GameMode::Inventory => {
                carried.clear();
                GameMode::Playing
            }
            GameMode::Paused => GameMode::Playing,
            GameMode::GeneratorSettings => {
                placement.generator_panel = None;
                GameMode::Playing
            }
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
            carried.clear();
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
            if placement.selected != index {
                placement.selection.clear();
                placement.edit_gesture = None;
                placement.selected = index;
            }
        }
    }

    if keys.just_pressed(bindings.rotate_or_rollback.key_code())
        && !simulation.is_active()
        && placement.target.is_none()
    {
        placement.facing = placement.facing.rotate();
    }
}

pub fn placement_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    inventory: Res<InventoryItems>,
    config: Res<GameConfig>,
    builder_mode: Res<BuilderMode>,
    mut mode: ResMut<GameMode>,
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

    if selected_area(&inventory, &placement) != Some(AreaKind::Selection) {
        placement.selection.clear();
    }

    let current_place_at = placement.target.map(|target| target.pos + target.normal);
    let current_delete_at = placement.target.map(|target| target.pos);
    let current_target_pos = placement.target.map(|target| target.pos);
    let can_preview_place = current_place_at.is_some_and(|pos| {
        selected_place_block(&inventory, *builder_mode, &placement)
            .is_some_and(|block| can_place_block_at(pos, block, *builder_mode, &world, &player))
    });

    if mouse_buttons.just_pressed(place_button)
        && selected_kind(&inventory, &placement).is_none()
        && current_target_pos.is_some_and(|pos| {
            world
                .blocks
                .get(&pos)
                .is_some_and(|block| block.kind == BlockKind::Generator)
        })
    {
        placement.generator_panel = current_target_pos;
        placement.edit_gesture = None;
        placement.selection.clear();
        *mode = GameMode::GeneratorSettings;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if selected_area(&inventory, &placement) == Some(AreaKind::Selection) {
        handle_selection_area_input(
            &mouse_buttons,
            current_target_pos,
            &mut placement,
            &mut world,
            &block_entities,
            &mut commands,
            &render_assets,
        );
        despawn_edit_previews(&mut commands, &edit_previews);
        spawn_selection_previews(&placement, &mut commands, &render_assets);
        return;
    }

    placement.selection.clear();

    if keys.just_pressed(config.key_bindings.rotate_or_rollback.key_code()) {
        if !rotate_pending_place_preview(&mut placement) {
            if can_preview_place {
                placement.facing = placement.facing.rotate();
            } else if let Some(pos) = current_target_pos {
                rotate_block_at(
                    pos,
                    &mut world,
                    &mut placement,
                    &block_entities,
                    &mut commands,
                    &render_assets,
                );
            }
        }
    }

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

fn rotate_pending_place_preview(placement: &mut PlacementState) -> bool {
    let Some(EditGesture {
        kind: EditGestureKind::Place { block },
        ..
    }) = placement.edit_gesture.as_mut()
    else {
        return false;
    };

    block.facing = block.facing.rotate();
    placement.facing = block.facing;
    true
}

fn selected_place_block(
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    placement: &PlacementState,
) -> Option<BlockData> {
    let kind = inventory.hotbar[placement.selected]?;
    let kind = kind.block()?;
    can_place_in_mode(kind, builder_mode).then_some(BlockData {
        kind,
        facing: placement.facing,
    })
}

fn selected_kind(inventory: &InventoryItems, placement: &PlacementState) -> Option<BlockKind> {
    inventory.hotbar[placement.selected].and_then(|item| item.block())
}

fn selected_area(inventory: &InventoryItems, placement: &PlacementState) -> Option<AreaKind> {
    inventory.hotbar[placement.selected].and_then(|item| item.area())
}

fn handle_selection_area_input(
    mouse_buttons: &ButtonInput<MouseButton>,
    current_target_pos: Option<IVec3>,
    placement: &mut PlacementState,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) {
    let place_button = MouseButton::Left;

    if let Some(drag) = placement.selection.drag.as_mut() {
        if let Some(current) = current_target_pos {
            if let Some((axis, offset)) = selection_drag_offset(*drag, current) {
                drag.axis = Some(axis);
                drag.offset = offset;
            }
        }
    }

    if mouse_buttons.just_released(place_button) {
        if let Some(drag) = placement.selection.drag.take() {
            if drag.offset != IVec3::ZERO {
                if let Some(bounds) = placement.selection.bounds {
                    if move_selection(
                        world,
                        block_entities,
                        commands,
                        render_assets,
                        bounds,
                        drag.offset,
                    ) {
                        placement.selection.bounds = Some(bounds.moved(drag.offset));
                    }
                }
            }
        }
        return;
    }

    if !mouse_buttons.just_pressed(place_button) {
        return;
    }

    let Some(pos) = current_target_pos else {
        return;
    };

    if let Some(bounds) = placement.selection.bounds {
        if bounds.contains(pos) {
            placement.selection.drag = Some(SelectionDrag {
                start: pos,
                axis: None,
                offset: IVec3::ZERO,
            });
            return;
        }
    }

    if let Some(first) = placement.selection.first_corner.take() {
        placement.selection.bounds = Some(SelectionBounds::from_corners(first, pos));
        placement.selection.drag = None;
    } else {
        placement.selection.first_corner = Some(pos);
        placement.selection.bounds = None;
        placement.selection.drag = None;
    }
}

fn selection_drag_offset(drag: SelectionDrag, current: IVec3) -> Option<(SelectionAxis, IVec3)> {
    let delta = current - drag.start;
    if delta == IVec3::ZERO {
        return None;
    }
    let axis = drag.axis.unwrap_or_else(|| strongest_axis(delta));
    let offset = match axis {
        SelectionAxis::X => axis.offset(delta.x),
        SelectionAxis::Y => axis.offset(delta.y),
        SelectionAxis::Z => axis.offset(delta.z),
    };
    Some((axis, offset))
}

fn strongest_axis(delta: IVec3) -> SelectionAxis {
    if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        SelectionAxis::X
    } else if delta.y.abs() >= delta.z.abs() {
        SelectionAxis::Y
    } else {
        SelectionAxis::Z
    }
}

fn move_selection(
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
    bounds: SelectionBounds,
    offset: IVec3,
) -> bool {
    let positions = bounds.positions();
    let selected: Vec<(IVec3, BlockData)> = positions
        .iter()
        .filter_map(|pos| world.blocks.get(pos).copied().map(|block| (*pos, block)))
        .collect();
    if selected.is_empty() {
        return true;
    }

    let selected_positions: std::collections::HashSet<IVec3> =
        selected.iter().map(|(pos, _)| *pos).collect();
    let updated_welds = moved_selection_welds(world, &selected_positions, offset);
    if selected.iter().any(|(pos, block)| {
        let target = *pos + offset;
        target.y < 0
            || (!selected_positions.contains(&target)
                && world
                    .blocks
                    .get(&target)
                    .is_some_and(|target_block| target_block.kind.has_collision()))
            || !block.kind.has_collision()
    }) {
        return false;
    }

    for (pos, _) in &selected {
        world.remove(pos);
        if let Some((entity, _)) = block_entities
            .iter()
            .find(|(_, block_entity)| block_entity.pos == *pos)
        {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (pos, block) in selected {
        let target = pos + offset;
        world.insert(target, block);
        spawn_block_with_animation(
            commands,
            render_assets,
            world,
            target,
            block,
            Some(BlockAnimation {
                from_pos: pos,
                to_pos: target,
                from_facing: block.facing,
                to_facing: block.facing,
            }),
        );
    }
    world.replace_material_welds(updated_welds);
    true
}

fn despawn_block_entities(commands: &mut Commands, block_entities: &Query<(Entity, &BlockEntity)>) {
    for (entity, _) in block_entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn refresh_edit_generated_markers(world: &mut WorldBlocks) {
    world.clear_generated_markers();
    let welders: Vec<(IVec3, crate::game::world::blocks::Facing)> = world
        .blocks
        .iter()
        .filter_map(|(pos, block)| {
            (block.kind == BlockKind::Welder).then_some((*pos, block.facing))
        })
        .collect();

    for (pos, facing) in welders {
        let point_pos = pos + facing.forward_ivec3();
        if !world.is_occupied(point_pos) {
            world.insert(
                point_pos,
                BlockData {
                    kind: BlockKind::WeldPoint,
                    facing,
                },
            );
        }
    }
}

fn rotate_block_at(
    pos: IVec3,
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) {
    let Some(block) = world.blocks.get_mut(&pos) else {
        return;
    };
    if !block.kind.is_directional() {
        return;
    }

    let from_facing = block.facing;
    block.facing = block.facing.rotate();
    let updated = *block;
    placement.facing = updated.facing;

    refresh_edit_generated_markers(world);
    let mut animations = std::collections::HashMap::new();
    animations.insert(
        pos,
        BlockAnimation {
            from_pos: pos,
            to_pos: pos,
            from_facing,
            to_facing: updated.facing,
        },
    );

    despawn_block_entities(commands, block_entities);
    rebuild_world_with_animations(commands, world, render_assets, &animations);
}

fn moved_selection_welds(
    world: &WorldBlocks,
    selected_positions: &std::collections::HashSet<IVec3>,
    offset: IVec3,
) -> std::collections::HashSet<MaterialWeld> {
    world
        .material_welds
        .iter()
        .filter_map(|weld| {
            let move_a = selected_positions.contains(&weld.a);
            let move_b = selected_positions.contains(&weld.b);
            if move_a != move_b {
                return None;
            }
            let a = if move_a { weld.a + offset } else { weld.a };
            let b = if move_b { weld.b + offset } else { weld.b };
            (a != b).then_some(MaterialWeld::new(a, b))
        })
        .collect()
}

fn spawn_selection_previews(
    placement: &PlacementState,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) {
    if let Some(first) = placement.selection.first_corner {
        spawn_edit_preview(commands, render_assets, first, EditPreviewKind::Selection);
    }

    if let Some(bounds) = placement.selection.bounds {
        let offset = placement
            .selection
            .drag
            .map(|drag| drag.offset)
            .unwrap_or(IVec3::ZERO);
        for pos in bounds.moved(offset).positions() {
            spawn_edit_preview(commands, render_assets, pos, EditPreviewKind::Selection);
        }
    }
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
                }
            }
            refresh_edit_generated_markers(world);
            despawn_block_entities(commands, block_entities);
            rebuild_world(commands, world, render_assets);
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
            refresh_edit_generated_markers(world);
            despawn_block_entities(commands, block_entities);
            rebuild_world(commands, world, render_assets);
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
            let positions: Vec<IVec3> = positions
                .into_iter()
                .filter(|pos| can_place_block_at(*pos, block, builder_mode, world, player))
                .collect();
            let preview_world = preview_world(world, &positions, block);
            for pos in positions {
                spawn_block_preview(commands, render_assets, &preview_world, pos, block);
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

fn preview_world(world: &WorldBlocks, positions: &[IVec3], block: BlockData) -> WorldBlocks {
    let mut preview = world.clone();
    for pos in positions {
        preview.insert(*pos, block);
    }
    refresh_edit_generated_markers(&mut preview);
    preview
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
