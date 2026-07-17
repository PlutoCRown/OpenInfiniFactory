//! 放置/删除合法性判定

use bevy::prelude::*;

use crate::game::blocks::BlockPresent;
use crate::game::blocks::{BlockData, BlockKind};
use crate::game::player::controller::player_intersects_block;
use crate::game::state::BuilderMode;
use crate::game::world::grid::WorldBlocks;

/// 判断指定位置在当前模式下是否允许放置该方块
pub(super) fn can_place_block_at(
    place_at: IVec3,
    block: BlockData,
    builder_mode: BuilderMode,
    world: &WorldBlocks,
    player_pos: Option<Vec3>,
    face_normal: Option<IVec3>,
) -> bool {
    if place_at.y < 0 {
        return false;
    }

    if !world.can_place_block_kind_at(place_at, block.kind) {
        return false;
    }

    if !can_place_in_mode(block.kind, builder_mode) {
        return false;
    }

    if block.kind == BlockKind::Sign {
        let Some(normal) = face_normal.filter(|n| n.abs().element_sum() == 1) else {
            return false;
        };
        if !world.can_place_sign_on_face(place_at - normal, normal) {
            return false;
        }
    }

    if let Some(position) = player_pos {
        if player_intersects_block(position, place_at) {
            return false;
        }
    }

    true
}

/// 判断当前建造模式下该方块种类是否可放置
pub(super) fn can_place_in_mode(kind: BlockKind, mode: BuilderMode) -> bool {
    match mode {
        BuilderMode::Edit => kind.is_editable(),
        BuilderMode::Play => kind.is_factory(),
    }
}

/// 判断指定位置在当前模式下是否可删除
pub(super) fn can_delete_at(pos: IVec3, mode: BuilderMode, world: &WorldBlocks) -> bool {
    match mode {
        // 编辑模式：有方块即可删（系统方块多为 no_collision，不能用 is_occupied）
        BuilderMode::Edit => {
            world.blocks.contains_key(&pos) || world.system_blocks.contains_key(&pos)
        }
        BuilderMode::Play => world
            .blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_factory()),
    }
}

/// 按建造模式删除指定位置的方块
pub(super) fn delete_block_at(pos: IVec3, mode: BuilderMode, world: &mut WorldBlocks) -> bool {
    match mode {
        BuilderMode::Edit => world.remove(&pos).is_some() || world.remove_system(&pos).is_some(),
        BuilderMode::Play => {
            if !world
                .blocks
                .get(&pos)
                .is_some_and(|block| block.kind.is_factory())
            {
                return false;
            }
            world.remove(&pos).is_some()
        }
    }
}
