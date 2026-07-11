//! 框选区域拖拽与预览

use bevy::prelude::*;

use crate::game::blocks::BlockData;
use crate::game::edit_history::{
    build_relocate_patch, capture_welds_for_ids, weld_diff, EditHistory,
};
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{
    PlacementState, SelectionAxis, SelectionBounds, SelectionDrag,
};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::BlockAnimation;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    rebuild_world_with_animations_for_debug_state, spawn_block_with_animation, spawn_edit_preview,
    BlockEntity, BlockEntityLayer, EditPreviewKind, WorldRenderAssets,
};
use crate::scene::BlockEntityIndex;
use crate::shared::config::ConfigSelectionMode;

use super::placement::despawn_block_entities;

/// 处理框选工具的点击与拖拽输入
pub(super) fn handle_selection_area_input(
    mouse_buttons: &ButtonInput<MouseButton>,
    current_target_pos: Option<IVec3>,
    place_button: MouseButton,
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
                        edit_history,
                        block_entities,
                        commands,
                        meshes,
                        render_assets,
                        debug,
                        structure_state,
                        block_index,
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

/// 根据拖拽起点与当前格计算轴向偏移
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

    let selected_ids: std::collections::HashSet<_> = selected
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

    // 焊接按 BlockId：整段一起搬时无需改写；只断掉选区边界焊缝
    let weld_count = world.material_welds.len();
    world
        .material_welds
        .retain(|weld| selected_ids.contains(&weld.a) == selected_ids.contains(&weld.b));
    if world.material_welds.len() != weld_count {
        world.topology_revision = world.topology_revision.wrapping_add(1);
    }

    for (pos, _) in &selected {
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

    let relocate_moves: Vec<(IVec3, IVec3, BlockData)> = selected
        .iter()
        .map(|(pos, block)| (*pos, *pos + offset, *block))
        .collect();
    world.relocate_blocks(relocate_moves);

    let welds_after = capture_welds_for_ids(world, &selected_ids);
    let (welds_add, welds_remove) = weld_diff(&welds_before, &welds_after);
    patch.welds_add = welds_add;
    patch.welds_remove = welds_remove;
    edit_history.record(patch);

    let mut animations = std::collections::HashMap::new();
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
    if debug.factory_activity {
        despawn_block_entities(commands, block_entities, block_index);
        rebuild_world_with_animations_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            &animations,
            debug,
            structure_state,
            block_index,
        );
    } else {
        for (target, animation) in animations {
            let block = world.blocks[&target];
            spawn_block_with_animation(
                commands,
                meshes,
                render_assets,
                world,
                target,
                block,
                Some(animation),
                None,
                block_index,
            );
        }
    }
    true
}

/// 生成框选区域的编辑预览方块
pub(super) fn spawn_selection_previews(
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

/// 按点/线/面模式展开起止格之间的选区位置
pub(super) fn selection_positions(mode: ConfigSelectionMode, start: IVec3, end: IVec3) -> Vec<IVec3> {
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
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}
