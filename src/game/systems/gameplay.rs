use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::game::player::controller::{player_intersects_block, FlyCamera};
use crate::game::state::{
    BuilderMode, EditGesture, EditGestureKind, GameMode, GameSettings, PlacementState,
    SelectionAxis, SelectionBounds, SelectionDrag, SimulationState, SolutionState,
    TeleportRenameState,
};
use crate::game::ui::{AreaKind, CarriedItem, InventoryItems, PendingKeyBind, HOTBAR_SLOTS};
use crate::game::world::animation::BlockAnimation;
use crate::game::world::blocks::{BlockData, BlockKind, MarkerBehavior, MaterialSource};
use crate::game::world::grid::{grid_to_world, raycast_blocks, MaterialWeld, WorldBlocks};
use crate::game::world::rendering::{
    despawn_edit_previews, rebuild_world, rebuild_world_with_animations, spawn_block_preview,
    spawn_block_with_animation, spawn_edit_preview, BlockEntity, EditPreview, EditPreviewKind,
    HoverMarker, PlacementPreview, WorldRenderAssets,
};
use crate::shared::config::{ConfigSelectionMode, GameConfig};

pub fn gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    config: Res<GameConfig>,
    simulation: Res<SimulationState>,
    pending_key_bind: Res<PendingKeyBind>,
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
    mut teleport_rename: ResMut<TeleportRenameState>,
    mut carried: ResMut<CarriedItem>,
) {
    let bindings = &config.key_bindings;

    if *mode == GameMode::Settings && pending_key_bind.0.is_some() {
        mouse_wheel.clear();
        return;
    }

    if keys.just_pressed(bindings.pause.key_code()) {
        if *mode == GameMode::TeleportSettings && teleport_rename.editing.is_some() {
            teleport_rename.editing = None;
            return;
        }
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
            GameMode::LabelerSettings => {
                placement.labeler_panel = None;
                GameMode::Playing
            }
            GameMode::ConverterSettings => {
                placement.converter_panel = None;
                GameMode::Playing
            }
            GameMode::TeleportSettings => {
                teleport_rename.editing = None;
                placement.teleport_panel = None;
                GameMode::Playing
            }
            GameMode::Settings => GameMode::Paused,
            GameMode::SaveListMain => GameMode::MainMenu,
            other => other,
        };
    }

    if !matches!(*mode, GameMode::Playing | GameMode::Inventory) {
        mouse_wheel.clear();
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

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
    if wheel_delta.abs() > f32::EPSILON {
        let direction = if wheel_delta > 0.0 { -1 } else { 1 };
        let selected = (placement.selected as i32 + direction).rem_euclid(HOTBAR_SLOTS as i32);
        if placement.selected != selected as usize {
            placement.selection.clear();
            placement.edit_gesture = None;
            placement.selected = selected as usize;
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
    mut solution_state: ResMut<SolutionState>,
    mut inventory: ResMut<InventoryItems>,
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
    let place_button = config
        .input(crate::shared::config::ConfigAction::Place)
        .mouse_button()
        .unwrap_or(MouseButton::Left);
    let delete_button = config
        .input(crate::shared::config::ConfigAction::Delete)
        .mouse_button()
        .unwrap_or(MouseButton::Right);
    let pick_button = config
        .input(crate::shared::config::ConfigAction::Pick)
        .mouse_button()
        .unwrap_or(MouseButton::Middle);

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

    let force_place = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if mouse_buttons.just_pressed(place_button)
        && !force_place
        && open_target_block_ui(current_target_pos, &world, &mut placement, &mut mode)
    {
        placement.edit_gesture = None;
        placement.selection.clear();
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if selected_area(&inventory, &placement) == Some(AreaKind::Selection) {
        if handle_selection_area_input(
            &mouse_buttons,
            current_target_pos,
            place_button,
            &mut placement,
            &mut world,
            &block_entities,
            &mut commands,
            &render_assets,
        ) {
            solution_state.dirty = true;
        }
        despawn_edit_previews(&mut commands, &edit_previews);
        spawn_selection_previews(&placement, &mut commands, &render_assets);
        return;
    }

    placement.selection.clear();

    if mouse_buttons.just_pressed(pick_button) {
        if let Some(pos) = current_target_pos {
            pick_target_block(pos, &world, &mut placement, &mut inventory);
        }
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if *builder_mode == BuilderMode::Play
        && !simulation.is_active()
        && keys.just_pressed(config.key_bindings.alternate.key_code())
    {
        if let Some(pos) = current_target_pos {
            if alternate_block_at(
                pos,
                &mut world,
                &block_entities,
                &mut commands,
                &render_assets,
            ) {
                solution_state.dirty = true;
            }
        }
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if keys.just_pressed(config.key_bindings.rotate_or_rollback.key_code()) {
        if !rotate_pending_place_preview(&mut placement) {
            if can_preview_place {
                placement.facing = placement.facing.rotate();
            } else if let Some(pos) = current_target_pos {
                if rotate_block_at(
                    pos,
                    &mut world,
                    &mut placement,
                    &block_entities,
                    &mut commands,
                    &render_assets,
                ) {
                    solution_state.dirty = true;
                }
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
                if commit_edit_gesture(
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
                ) {
                    solution_state.dirty = true;
                }
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

fn selected_area(inventory: &InventoryItems, placement: &PlacementState) -> Option<AreaKind> {
    inventory.hotbar[placement.selected].and_then(|item| item.area())
}

fn open_target_block_ui(
    target: Option<IVec3>,
    world: &WorldBlocks,
    placement: &mut PlacementState,
    mode: &mut GameMode,
) -> bool {
    let Some(pos) = target else {
        return false;
    };
    let Some(block) = world.system_blocks.get(&pos) else {
        return false;
    };

    if matches!(
        block.kind.material_source(block.facing),
        Some(MaterialSource::Generator)
    ) {
        placement.generator_panel = Some(pos);
        *mode = GameMode::GeneratorSettings;
        return true;
    }
    if block.kind.material_labeler(block.facing).is_some() {
        placement.labeler_panel = Some(pos);
        *mode = GameMode::LabelerSettings;
        return true;
    }
    if block.kind == BlockKind::Converter {
        placement.converter_panel = Some(pos);
        *mode = GameMode::ConverterSettings;
        return true;
    }
    if block.kind.is_teleport() {
        placement.teleport_panel = Some(pos);
        *mode = GameMode::TeleportSettings;
        return true;
    }
    false
}

fn pick_target_block(
    pos: IVec3,
    world: &WorldBlocks,
    placement: &mut PlacementState,
    inventory: &mut InventoryItems,
) {
    let Some(kind) = world
        .blocks
        .get(&pos)
        .or_else(|| world.system_blocks.get(&pos))
        .map(|block| block.kind)
    else {
        return;
    };
    if !inventory.can_take_block(kind) {
        return;
    }

    if let Some(index) = inventory.hotbar_index_of_block(kind) {
        placement.selected = index;
    } else {
        inventory.set_hotbar_block(placement.selected, kind);
    }
    placement.selection.clear();
    placement.edit_gesture = None;
}

fn handle_selection_area_input(
    mouse_buttons: &ButtonInput<MouseButton>,
    current_target_pos: Option<IVec3>,
    place_button: MouseButton,
    placement: &mut PlacementState,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) -> bool {
    let mut changed = false;
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
                        changed = true;
                    }
                }
            }
        }
        return changed;
    }

    if !mouse_buttons.just_pressed(place_button) {
        return false;
    }

    let Some(pos) = current_target_pos else {
        return false;
    };

    if let Some(bounds) = placement.selection.bounds {
        if bounds.contains(pos) {
            placement.selection.drag = Some(SelectionDrag {
                start: pos,
                axis: None,
                offset: IVec3::ZERO,
            });
            return false;
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
    false
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
            commands.entity(entity).despawn();
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
                kind: crate::game::world::animation::BlockAnimationKind::Move,
                duration: None,
                progress: None,
            }),
        );
    }
    world.replace_material_welds(updated_welds);
    true
}

fn despawn_block_entities(commands: &mut Commands, block_entities: &Query<(Entity, &BlockEntity)>) {
    for (entity, _) in block_entities {
        commands.entity(entity).despawn();
    }
}

fn refresh_edit_generated_markers(world: &mut WorldBlocks) {
    world.clear_generated_markers();
    let weld_points: Vec<(IVec3, IVec3, crate::game::world::direction::Facing)> = world
        .blocks
        .iter()
        .filter_map(
            |(pos, block)| match block.kind.marker_behavior(block.facing) {
                Some(MarkerBehavior::WeldPoint { offset, facing }) => Some((*pos, offset, facing)),
                _ => None,
            },
        )
        .collect();

    for (pos, offset, facing) in weld_points {
        let point_pos = pos + offset;
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

fn alternate_block_at(
    pos: IVec3,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) -> bool {
    let Some(block) = world.blocks.get_mut(&pos) else {
        return false;
    };
    let Some(kind) = block.kind.alternate() else {
        return false;
    };

    block.kind = kind;
    refresh_edit_generated_markers(world);
    despawn_block_entities(commands, block_entities);
    rebuild_world(commands, world, render_assets);
    true
}

fn rotate_block_at(
    pos: IVec3,
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) -> bool {
    let Some(block) = world.blocks.get_mut(&pos) else {
        return false;
    };
    if !block.kind.is_directional() {
        return false;
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
            kind: crate::game::world::animation::BlockAnimationKind::Move,
            duration: None,
            progress: None,
        },
    );

    despawn_block_entities(commands, block_entities);
    rebuild_world_with_animations(commands, world, render_assets, &animations);
    true
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
    if place_at.y < 0 {
        return false;
    }

    if block.kind.is_system_layer() {
        if world.system_blocks.contains_key(&place_at) {
            return false;
        }
    } else if !world.can_place_solid_at(place_at) {
        return false;
    }

    if !can_place_in_mode(block.kind, builder_mode) {
        return false;
    }

    if let Ok(player_transform) = player.single() {
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
) -> bool {
    let mut changed = false;
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
                    changed = true;
                }
            }
            if changed {
                refresh_edit_generated_markers(world);
                despawn_block_entities(commands, block_entities);
                rebuild_world(commands, world, render_assets);
            }
        }
        EditGestureKind::Delete => {
            let positions = selection_positions(
                config.delete_selection_mode,
                gesture.start,
                current_delete_at.unwrap_or(gesture.start),
            );
            for pos in positions {
                let removed = world.remove(&pos).is_some() || world.remove_system(&pos).is_some();
                if removed {
                    changed = true;
                    if let Some((entity, _)) = block_entities
                        .iter()
                        .find(|(_, block_entity)| block_entity.pos == pos)
                    {
                        commands.entity(entity).despawn();
                    }
                }
            }
            if changed {
                refresh_edit_generated_markers(world);
                despawn_block_entities(commands, block_entities);
                rebuild_world(commands, world, render_assets);
            }
        }
    }
    changed
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
        (&mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>),
        (With<HoverMarker>, Without<FlyCamera>),
    >,
    mut preview: Query<
        (&mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>),
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
        if let Ok((_, mut visibility, _)) = marker.single_mut() {
            *visibility = Visibility::Hidden;
        }
        if let Ok((_, mut visibility, _)) = preview.single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Ok(camera_transform) = camera.single() else {
        return;
    };

    placement.target = raycast_blocks(
        camera_transform.translation,
        *camera_transform.forward(),
        &world,
    );

    let Ok((mut marker_transform, mut marker_visibility, marker_material)) =
        marker.single_mut()
    else {
        return;
    };

    if placement.edit_gesture.is_some() {
        *marker_visibility = Visibility::Hidden;
    } else if let Some(target) = placement.target {
        marker_transform.translation = grid_to_world(target.pos);
        *marker_visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(&marker_material.0) {
            material.base_color = Color::srgba(1.0, 1.0, 1.0, 0.16);
        }
    } else {
        *marker_visibility = Visibility::Hidden;
    }

    let Ok((_, mut preview_visibility, _)) = preview.single_mut() else {
        return;
    };

    let _ = (inventory, builder_mode, player);
    *preview_visibility = Visibility::Hidden;
}

fn can_place_in_mode(kind: BlockKind, mode: BuilderMode) -> bool {
    match mode {
        BuilderMode::Edit => kind.is_editable(),
        BuilderMode::Play => kind.is_factory(),
    }
}
