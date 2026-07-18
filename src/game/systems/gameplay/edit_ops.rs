//! 取块/切换变体/旋转

use bevy::prelude::*;

use crate::game::blocks::BlockKind;
use crate::game::edit_history::{build_cell_patch, EditHistory};
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::PlacementState;
use crate::game::systems::debug::DebugState;
use crate::game::ui::InventoryItems;
use crate::game::world::animation::BlockAnimation;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    rebuild_world_for_debug_state, rebuild_world_with_animations,
    rebuild_world_with_animations_for_debug_state, BlockEntity,
};
use crate::game::world::rendering::WorldRenderAssets;
use crate::scene::BlockEntityIndex;

use super::placement::{despawn_block_entities, refresh_edit_generated_markers};

/// 从准星目标拾取方块到快捷栏
pub(super) fn pick_target_block(
    pos: IVec3,
    world: &WorldBlocks,
    placement: &mut PlacementState,
    inventory: &mut InventoryItems,
) {
    let Some(block) = world
        .blocks
        .get(&pos)
        .or_else(|| world.system_blocks.get(&pos))
    else {
        return;
    };
    let kind = block.kind;
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

/// 切换目标方块的变体并重建场景
pub(super) fn alternate_block_at(
    pos: IVec3,
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
    let patch = build_cell_patch(world, &[pos], |world| {
        let Some(block) = world.blocks.get_mut(&pos) else {
            return;
        };
        let Some(kind) = block.kind.alternate() else {
            return;
        };

        if matches!(
            (block.kind, kind),
            (BlockKind::Conveyor, BlockKind::ReverseConveyor)
                | (BlockKind::ReverseConveyor, BlockKind::Conveyor)
        ) {
            block.facing = block.facing.rotate().rotate();
        }
        block.kind = kind;
    });
    if patch.is_empty() {
        return false;
    }
    edit_history.record(patch);
    refresh_edit_generated_markers(world);
    despawn_block_entities(commands, block_entities, block_index);
    rebuild_world_for_debug_state(
        commands,
        meshes,
        world,
        render_assets,
        debug,
        structure_state,
        block_index,
    );
    true
}

/// 旋转目标方块朝向并重建场景
pub(super) fn rotate_block_at(
    pos: IVec3,
    reverse: bool,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
) -> bool {
    let in_system = !world.blocks.contains_key(&pos);
    let Some(block) = (if in_system {
        world.system_blocks.get_mut(&pos)
    } else {
        world.blocks.get_mut(&pos)
    }) else {
        return false;
    };
    if !block.kind.is_directional() {
        return false;
    }

    let from_facing = block.facing;
    block.facing = rotate_facing(block.facing, reverse);
    let updated = *block;

    refresh_edit_generated_markers(world);
    let mut animations = std::collections::HashMap::new();
    animations.insert(
        pos,
        BlockAnimation {
            block_id: updated.id,
            from_pos: pos,
            to_pos: pos,
            from_facing,
            to_facing: updated.facing,
            kind: crate::game::world::animation::BlockAnimationKind::Move,
            duration: None,
            progress: None,
        },
    );

    despawn_block_entities(commands, block_entities, block_index);
    if debug.factory_activity {
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
        rebuild_world_with_animations(
            commands,
            meshes,
            world,
            render_assets,
            &animations,
            None,
            block_index,
        );
    }
    true
}

/// 按正/反方向旋转朝向
pub(super) fn rotate_facing(
    facing: crate::game::blocks::Facing,
    reverse: bool,
) -> crate::game::blocks::Facing {
    if reverse {
        facing.rotate_counter()
    } else {
        facing.rotate()
    }
}

/// 是否按住 Shift
pub(super) fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}
