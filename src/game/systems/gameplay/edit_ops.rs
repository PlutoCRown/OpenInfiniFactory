//! 取块/切换变体/旋转

use bevy::prelude::*;

use crate::game::edit_history::build_cell_patch;
use crate::game::state::PlacementState;
use crate::game::ui::InventoryItems;
use crate::game::world::animation::BlockAnimation;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{rebuild_world_for_debug_state, rebuild_world_with_animations};
use crate::scene::WorldEditScene;

use super::placement::{despawn_block_entities, refresh_edit_generated_markers};
use super::rules::can_manual_rotate;

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
    // 中键取有向块时，放置朝向跟它对齐（有向附着物朝向由贴面决定，不写入）
    if can_manual_rotate(kind) {
        placement.preview_facing = block.facing;
    }
    placement.selection.clear();
    placement.edit_gesture = None;
}

/// 切换目标方块的变体并重建场景
pub(super) fn alternate_block_at(edit: &mut WorldEditScene, pos: IVec3) -> bool {
    let patch = build_cell_patch(edit.world, &[pos], |world| {
        let Some(block) = world.blocks.get_mut(&pos) else {
            return;
        };
        let Some(kind) = block.kind.alternate() else {
            return;
        };

        if block.kind.alternate_flip_facing() {
            block.facing = block.facing.rotate().rotate();
        }
        block.kind = kind;
    });
    if patch.is_empty() {
        return false;
    }
    edit.edit_history.record(patch);
    refresh_edit_generated_markers(edit.world);
    edit.scene
        .structure_state
        .apply_factory_edit(edit.world, &std::collections::HashSet::from([pos]));
    despawn_block_entities(
        edit.scene.commands,
        edit.scene.meshes,
        edit.block_entities,
        edit.scene.block_index,
        edit.scene.scene_chunks,
    );
    rebuild_world_for_debug_state(
        edit.scene.commands,
        edit.scene.meshes,
        edit.world,
        edit.scene.render_assets,
        edit.scene.debug,
        edit.scene.structure_state,
        edit.scene.block_index,
        edit.scene.scene_chunks,
    );
    true
}

/// 旋转目标方块朝向并重建场景
pub(super) fn rotate_block_at(edit: &mut WorldEditScene, pos: IVec3, reverse: bool) -> bool {
    let in_system = !edit.world.blocks.contains_key(&pos);
    let Some(block) = (if in_system {
        edit.world.system_blocks.get_mut(&pos)
    } else {
        edit.world.blocks.get_mut(&pos)
    }) else {
        return false;
    };
    if !can_manual_rotate(block.kind) {
        return false;
    }

    let from_facing = block.facing;
    block.facing = rotate_facing(block.facing, reverse);
    let updated = *block;

    refresh_edit_generated_markers(edit.world);
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

    edit.scene
        .structure_state
        .apply_factory_edit(edit.world, &std::collections::HashSet::from([pos]));
    despawn_block_entities(
        edit.scene.commands,
        edit.scene.meshes,
        edit.block_entities,
        edit.scene.block_index,
        edit.scene.scene_chunks,
    );
    rebuild_world_with_animations(
        edit.scene.commands,
        edit.scene.meshes,
        edit.world,
        edit.scene.render_assets,
        &animations,
        None,
        edit.scene.block_index,
        edit.scene.scene_chunks,
    );
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

pub(super) fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}
