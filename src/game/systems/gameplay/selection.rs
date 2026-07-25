//! 框选区域拖拽与预览

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::blocks::{BlockData, BlockId};
use crate::game::edit_history::{
    EditHistory, build_cell_patch, build_relocate_patch, capture_welds_for_ids, weld_diff,
};
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{
    BuilderMode, EditGestureKind, GameMode, PlacementState, PlayingUiState, SelectionAxis,
    SelectionBounds, SelectionDrag, SelectionSnapshot, SimulationState,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::{AreaKind, InventoryItems, UiRuntime};
use crate::game::world::animation::BlockAnimation;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    BlockEntity, BlockEntityLayer, DeleteBoundsOverlay, DeleteBoundsPart, SceneChunkMeshes,
    SelectionBoundsOverlay, SelectionBoundsPart, WorldRenderAssets,
    rebuild_world_with_animations_for_debug_state, spawn_block_with_animation,
    sync_scene_chunks_for_positions, update_delete_bounds_overlay, update_selection_bounds_overlay,
};
use crate::scene::BlockEntityIndex;
use crate::shared::config::{ConfigChord, ConfigSelectionMode, GameConfig};

use super::placement::despawn_block_entities;
use super::rules::can_delete_at;

/// 处理框选工具的点击与拖拽输入
pub(super) fn handle_selection_area_input(
    mouse_buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    current_target_pos: Option<IVec3>,
    place_button: MouseButton,
    delete_button: MouseButton,
    copy_chord: ConfigChord,
    force_place: bool,
    builder_mode: BuilderMode,
    placement: &mut PlacementState,
    world: &mut WorldBlocks,
    edit_history: &mut EditHistory,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
) -> bool {
    let mut changed = false;

    if mouse_buttons.just_pressed(delete_button) {
        if placement.selection.is_active() {
            let before = SelectionSnapshot::from_state(&placement.selection);
            placement.selection.clear();
            let after = SelectionSnapshot::from_state(&placement.selection);
            edit_history.record_with_selection(Default::default(), before, after);
        }
        return false;
    }

    if let Some(drag) = placement.selection.drag.as_mut() {
        if let Some(current) = current_target_pos {
            drag.offset = selection_drag_offset(*drag, current);
        }
    }

    // 拖动中按复制键：立即按当前偏移复制，并结束本次拖动
    if copy_chord.just_triggered(keys) {
        if let Some(drag) = placement.selection.drag.take() {
            if drag.offset != IVec3::ZERO {
                if let Some(bounds) = placement.selection.bounds {
                    let before = SelectionSnapshot::from_state(&placement.selection);
                    if copy_selection(
                        world,
                        edit_history,
                        block_entities,
                        commands,
                        meshes,
                        render_assets,
                        debug,
                        structure_state,
                        block_index,
                        scene_chunks,
                        bounds,
                        drag.offset,
                        force_place,
                        builder_mode,
                        &before,
                    ) {
                        placement.selection.bounds = Some(bounds.moved(drag.offset));
                        changed = true;
                    } else {
                        placement.selection.drag = Some(drag);
                    }
                }
            } else {
                placement.selection.drag = Some(drag);
            }
            return changed;
        }
    }

    if mouse_buttons.just_released(place_button) {
        if let Some(drag) = placement.selection.drag.take() {
            if drag.offset != IVec3::ZERO {
                if let Some(bounds) = placement.selection.bounds {
                    let before = SelectionSnapshot::from_state(&placement.selection);
                    if move_selection(
                        world,
                        edit_history,
                        block_entities,
                        commands,
                        meshes,
                        render_assets,
                        debug,
                        structure_state,
                        block_index,
                        scene_chunks,
                        bounds,
                        drag.offset,
                        force_place,
                        builder_mode,
                        &before,
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
                offset: IVec3::ZERO,
            });
            return false;
        }
        let before = SelectionSnapshot::from_state(&placement.selection);
        placement.selection.clear();
        let after = SelectionSnapshot::from_state(&placement.selection);
        edit_history.record_with_selection(Default::default(), before, after);
        return false;
    }

    if let Some(first) = placement.selection.first_corner.take() {
        let before = SelectionSnapshot {
            first_corner: Some(first),
            bounds: None,
        };
        placement.selection.bounds = Some(SelectionBounds::from_corners(first, pos));
        placement.selection.drag = None;
        let after = SelectionSnapshot::from_state(&placement.selection);
        edit_history.record_with_selection(Default::default(), before, after);
    } else {
        placement.selection.first_corner = Some(pos);
        placement.selection.bounds = None;
        placement.selection.drag = None;
    }
    false
}

/// 根据拖拽起点与当前格计算轴向偏移（每帧取当前最强轴，与放方块线选一致）
fn selection_drag_offset(drag: SelectionDrag, current: IVec3) -> IVec3 {
    let delta = current - drag.start;
    if delta == IVec3::ZERO {
        return IVec3::ZERO;
    }
    let axis = strongest_axis(delta);
    match axis {
        SelectionAxis::X => axis.offset(delta.x),
        SelectionAxis::Y => axis.offset(delta.y),
        SelectionAxis::Z => axis.offset(delta.z),
    }
}

/// 取位移绝对值最大的轴
fn strongest_axis(delta: IVec3) -> SelectionAxis {
    if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        SelectionAxis::X
    } else if delta.y.abs() >= delta.z.abs() {
        SelectionAxis::Y
    } else {
        SelectionAxis::Z
    }
}

/// 收集选区内可移动/可复制的方块（游玩模式仅工厂）
fn selected_blocks(
    world: &WorldBlocks,
    bounds: SelectionBounds,
    mode: BuilderMode,
) -> Vec<(IVec3, BlockData)> {
    bounds
        .positions()
        .iter()
        .filter_map(|pos| {
            let block = world.blocks.get(pos).copied()?;
            match mode {
                BuilderMode::Play => block.kind.is_factory().then_some((*pos, block)),
                BuilderMode::Edit => Some((*pos, block)),
            }
        })
        .collect()
}

/// 选区落点是否允许；force 为 Shift 强制覆盖
fn selection_can_place(
    world: &WorldBlocks,
    selected: &[(IVec3, BlockData)],
    offset: IVec3,
    vacate_sources: bool,
    mode: BuilderMode,
    force: bool,
) -> bool {
    let selected_positions: HashSet<IVec3> = selected.iter().map(|(pos, _)| *pos).collect();
    selected.iter().all(|(pos, block)| {
        let target = *pos + offset;
        if target.y < 0 || !block.kind.has_collision() {
            return false;
        }
        if vacate_sources && selected_positions.contains(&target) {
            return true;
        }
        if world.has_system_block_at(target) || world.has_generated_marker_at(target) {
            return false;
        }
        let Some(occupant) = world.blocks.get(&target) else {
            return true;
        };
        if !force {
            return !occupant.kind.has_collision();
        }
        match mode {
            BuilderMode::Edit => true,
            BuilderMode::Play => occupant.kind.is_factory(),
        }
    })
}

/// 需要被强制覆盖清掉的目标格（不含选区内即将腾空的格）
fn selection_overwrite_targets(
    world: &WorldBlocks,
    selected: &[(IVec3, BlockData)],
    offset: IVec3,
    vacate_sources: bool,
) -> Vec<IVec3> {
    let selected_positions: HashSet<IVec3> = selected.iter().map(|(pos, _)| *pos).collect();
    let mut targets = Vec::new();
    for (pos, _) in selected {
        let target = *pos + offset;
        if vacate_sources && selected_positions.contains(&target) {
            continue;
        }
        if world.blocks.contains_key(&target) {
            targets.push(target);
        }
    }
    targets.sort_by_key(|p| (p.x, p.y, p.z));
    targets.dedup();
    targets
}

/// 销毁指定格的方块实体
fn despawn_blocks_at(
    positions: &[IVec3],
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    block_index: &mut BlockEntityIndex,
) {
    for pos in positions {
        if let Some((entity, block_entity)) = block_entities
            .iter()
            .find(|(_, block_entity)| block_entity.pos == *pos)
        {
            match block_entity.layer {
                BlockEntityLayer::Animatable => {
                    block_index.remove_animatable(*pos);
                }
                BlockEntityLayer::System => {
                    block_index.remove_system(*pos);
                }
                BlockEntityLayer::Scene => {
                    block_index.remove_scene(*pos);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

/// 将框选区域内的方块整体平移
fn move_selection(
    world: &mut WorldBlocks,
    edit_history: &mut EditHistory,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
    bounds: SelectionBounds,
    offset: IVec3,
    force: bool,
    builder_mode: BuilderMode,
    selection_before: &SelectionSnapshot,
) -> bool {
    let selected = selected_blocks(world, bounds, builder_mode);
    if selected.is_empty() {
        let mut after = selection_before.clone();
        after.bounds = Some(bounds.moved(offset));
        edit_history.record_with_selection(Default::default(), selection_before.clone(), after);
        return true;
    }

    if !selection_can_place(world, &selected, offset, true, builder_mode, force) {
        return false;
    }

    let overwrite = if force {
        selection_overwrite_targets(world, &selected, offset, true)
    } else {
        Vec::new()
    };

    let selected_ids: HashSet<_> = selected
        .iter()
        .map(|(_, block)| block.id)
        .filter(|id| !id.is_none())
        .collect();
    let welds_before = capture_welds_for_ids(world, &selected_ids);
    let moves: Vec<(IVec3, IVec3)> = selected
        .iter()
        .map(|(pos, _)| (*pos, *pos + offset))
        .collect();
    let mut patch = build_relocate_patch(world, &moves);

    let weld_count = world.material_welds.len();
    world
        .material_welds
        .retain(|weld| selected_ids.contains(&weld.a) == selected_ids.contains(&weld.b));
    if world.material_welds.len() != weld_count {
        world.topology_revision = world.topology_revision.wrapping_add(1);
    }

    let mut despawn_positions: Vec<IVec3> = selected.iter().map(|(pos, _)| *pos).collect();
    despawn_positions.extend(overwrite.iter().copied());
    despawn_blocks_at(&despawn_positions, block_entities, commands, block_index);

    for pos in &overwrite {
        world.remove(pos);
    }

    let relocate_moves: Vec<(IVec3, IVec3, BlockData)> = selected
        .iter()
        .map(|(pos, block)| (*pos, *pos + offset, *block))
        .collect();
    world.relocate_blocks(relocate_moves);

    let welds_after = capture_welds_for_ids(world, &selected_ids);
    let (welds_add, welds_remove) = weld_diff(&welds_before, &welds_after);
    patch.welds_add = welds_add;
    patch.welds_remove = welds_remove;
    let mut after = selection_before.clone();
    after.bounds = Some(bounds.moved(offset));
    edit_history.record_with_selection(patch, selection_before.clone(), after);

    let mut animations = HashMap::new();
    for (pos, block) in selected {
        let target = pos + offset;
        let stored = world.blocks[&target];
        animations.insert(
            target,
            BlockAnimation {
                block_id: stored.id,
                from_pos: pos,
                to_pos: target,
                from_facing: block.facing,
                to_facing: block.facing,
                kind: crate::game::world::animation::BlockAnimationKind::Move,
                duration: None,
                progress: None,
            },
        );
    }
    let mut also_dirty = despawn_positions;
    also_dirty.extend(animations.keys().copied());
    spawn_selection_result(
        world,
        block_entities,
        commands,
        meshes,
        render_assets,
        debug,
        structure_state,
        block_index,
        scene_chunks,
        animations,
        also_dirty,
    );
    true
}

/// 将框选区域内的方块复制到偏移位置
fn copy_selection(
    world: &mut WorldBlocks,
    edit_history: &mut EditHistory,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
    bounds: SelectionBounds,
    offset: IVec3,
    force: bool,
    builder_mode: BuilderMode,
    selection_before: &SelectionSnapshot,
) -> bool {
    let selected = selected_blocks(world, bounds, builder_mode);
    if selected.is_empty() {
        let mut after = selection_before.clone();
        after.bounds = Some(bounds.moved(offset));
        edit_history.record_with_selection(Default::default(), selection_before.clone(), after);
        return true;
    }

    if !selection_can_place(world, &selected, offset, false, builder_mode, force) {
        return false;
    }

    let overwrite = if force {
        selection_overwrite_targets(world, &selected, offset, false)
    } else {
        Vec::new()
    };

    let selected_ids: HashSet<_> = selected
        .iter()
        .map(|(_, block)| block.id)
        .filter(|id| !id.is_none())
        .collect();
    let internal_welds: Vec<_> = world
        .material_welds
        .iter()
        .copied()
        .filter(|weld| selected_ids.contains(&weld.a) && selected_ids.contains(&weld.b))
        .collect();
    let settings_by_pos: HashMap<IVec3, _> = selected
        .iter()
        .filter_map(|(pos, _)| {
            world
                .block_settings
                .get(pos)
                .cloned()
                .map(|settings| (*pos, settings))
        })
        .collect();
    let target_positions: Vec<IVec3> = selected.iter().map(|(pos, _)| *pos + offset).collect();

    despawn_blocks_at(&overwrite, block_entities, commands, block_index);

    let patch = build_cell_patch(world, &target_positions, |world| {
        for pos in &overwrite {
            world.remove(pos);
        }
        let mut id_map = HashMap::new();
        for (pos, block) in &selected {
            let old_id = block.id;
            let mut copy = *block;
            copy.id = BlockId::NONE;
            let target = *pos + offset;
            world.insert(target, copy);
            let new_id = world.blocks[&target].id;
            if !old_id.is_none() {
                id_map.insert(old_id, new_id);
            }
            if let Some(settings) = settings_by_pos.get(pos) {
                world.block_settings.insert(target, settings.clone());
            }
        }
        let mut added_weld = false;
        for weld in &internal_welds {
            let Some(&a) = id_map.get(&weld.a) else {
                continue;
            };
            let Some(&b) = id_map.get(&weld.b) else {
                continue;
            };
            if world
                .material_welds
                .insert(crate::game::world::grid::MaterialWeld::new(a, b))
            {
                added_weld = true;
            }
        }
        if added_weld {
            world.topology_revision = world.topology_revision.wrapping_add(1);
        }
    });

    let mut after = selection_before.clone();
    after.bounds = Some(bounds.moved(offset));
    edit_history.record_with_selection(patch, selection_before.clone(), after);

    let mut animations = HashMap::new();
    for (pos, block) in selected {
        let target = pos + offset;
        let stored = world.blocks[&target];
        animations.insert(
            target,
            BlockAnimation {
                block_id: stored.id,
                from_pos: pos,
                to_pos: target,
                from_facing: block.facing,
                to_facing: block.facing,
                kind: crate::game::world::animation::BlockAnimationKind::Move,
                duration: None,
                progress: None,
            },
        );
    }
    let mut also_dirty = overwrite;
    also_dirty.extend(target_positions.iter().copied());
    spawn_selection_result(
        world,
        block_entities,
        commands,
        meshes,
        render_assets,
        debug,
        structure_state,
        block_index,
        scene_chunks,
        animations,
        also_dirty,
    );
    true
}

/// 选区搬移/复制后刷新实体
fn spawn_selection_result(
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
    animations: HashMap<IVec3, BlockAnimation>,
    also_dirty: impl IntoIterator<Item = IVec3>,
) {
    let mut scene_dirty: HashSet<IVec3> = also_dirty.into_iter().collect();
    if debug.factory_activity {
        despawn_block_entities(commands, meshes, block_entities, block_index, scene_chunks);
        rebuild_world_with_animations_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            &animations,
            debug,
            structure_state,
            block_index,
            scene_chunks,
        );
    } else {
        for (target, animation) in &animations {
            let block = world.blocks[target];
            if block.kind.is_scene() {
                scene_dirty.insert(*target);
                scene_dirty.insert(animation.from_pos);
                continue;
            }
            spawn_block_with_animation(
                commands,
                meshes,
                render_assets,
                world,
                *target,
                block,
                Some(*animation),
                None,
                block_index,
            );
        }
        if !scene_dirty.is_empty() {
            sync_scene_chunks_for_positions(
                commands,
                meshes,
                world,
                render_assets,
                scene_chunks,
                scene_dirty,
            );
        }
    }
}

/// 根据当前框选状态算出选区包围盒显示参数（min, max, include_frame, valid）
pub(super) fn selection_bounds_show(
    placement: &PlacementState,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    force_place: bool,
) -> Option<(IVec3, IVec3, bool, bool)> {
    if let Some(first) = placement.selection.first_corner {
        return Some((first, first, false, true));
    }

    let bounds = placement.selection.bounds?;
    let offset = placement
        .selection
        .drag
        .map(|drag| drag.offset)
        .unwrap_or(IVec3::ZERO);
    let moved = bounds.moved(offset);
    let valid = if offset == IVec3::ZERO {
        true
    } else {
        let selected = selected_blocks(world, bounds, builder_mode);
        selected.is_empty()
            || selection_can_place(world, &selected, offset, true, builder_mode, force_place)
    };
    Some((moved.min, moved.max, true, valid))
}

/// 按点/线/面模式展开起止格之间的选区位置
pub(super) fn selection_positions(
    mode: ConfigSelectionMode,
    start: IVec3,
    end: IVec3,
) -> Vec<IVec3> {
    match mode {
        ConfigSelectionMode::Point => vec![start],
        ConfigSelectionMode::Line => line_selection(start, end),
        ConfigSelectionMode::Plane => plane_selection(start, end),
    }
}

/// 沿主轴展开线选位置
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

/// 在水平面上展开面选位置
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

/// 返回两整数的有序最小最大值
fn min_max(a: i32, b: i32) -> (i32, i32) {
    if a <= b { (a, b) } else { (b, a) }
}

/// 每帧同步全局选区/删除包围盒（平移与显隐，不重建实体）
pub fn sync_edit_bounds_overlays(
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    simulation: Res<SimulationState>,
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    config: Res<GameConfig>,
    world: Res<WorldBlocks>,
    keys: Res<ButtonInput<KeyCode>>,
    render_assets: Option<Res<WorldRenderAssets>>,
    mut selection_parts: Query<
        (
            &SelectionBoundsPart,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (With<SelectionBoundsOverlay>, Without<DeleteBoundsOverlay>),
    >,
    mut delete_parts: Query<
        (&DeleteBoundsPart, &mut Transform, &mut Visibility),
        (With<DeleteBoundsOverlay>, Without<SelectionBoundsOverlay>),
    >,
) {
    let Some(assets) = render_assets.as_ref() else {
        return;
    };

    if *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || ui_runtime.blocks_gameplay()
        || simulation.is_active()
    {
        update_selection_bounds_overlay(&mut selection_parts, assets, None);
        update_delete_bounds_overlay(&mut delete_parts, None);
        return;
    }

    let selection_tool = inventory.hotbar[placement.selected].and_then(|item| item.area())
        == Some(AreaKind::Selection);

    if selection_tool {
        let force_place = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        let show = selection_bounds_show(&placement, &world, *builder_mode, force_place);
        update_selection_bounds_overlay(&mut selection_parts, assets, show);
        update_delete_bounds_overlay(&mut delete_parts, None);
        return;
    }

    update_selection_bounds_overlay(&mut selection_parts, assets, None);

    let delete_show = placement.edit_gesture.as_ref().and_then(|gesture| {
        if gesture.canceled {
            return None;
        }
        match gesture.kind {
            EditGestureKind::Delete => {
                let end = placement
                    .target
                    .map(|target| target.pos)
                    .unwrap_or(gesture.start);
                let positions =
                    selection_positions(config.delete_selection_mode, gesture.start, end);
                let deletable: Vec<IVec3> = positions
                    .into_iter()
                    .filter(|pos| can_delete_at(*pos, *builder_mode, &world))
                    .collect();
                SelectionBounds::from_positions(&deletable).map(|bounds| (bounds.min, bounds.max))
            }
            _ => None,
        }
    });
    update_delete_bounds_overlay(&mut delete_parts, delete_show);
}
