//! 瞄准派生上下文：由 placement.target + 世界解析，供状态栏/名牌等订阅

use bevy::prelude::*;

use crate::game::blocks::{BlockId, BlockKind};
use crate::game::simulation::structure_state::{StructureId, StructureState};
use crate::game::state::PlacementState;
use crate::game::world::direction::Facing;
use crate::game::world::grid::{TargetHit, WorldBlocks};

/// 瞄准格上的方块摘要（系统层与材料/工厂/场景共用）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AimBlockInfo {
    pub kind: BlockKind,
    pub facing: Facing,
    pub id: BlockId,
    /// 是否来自 system_blocks（Touch 配置 HUD 等可据此判断）
    pub system_layer: bool,
}

/// 当前瞄准派生结果；内容变了才写入，消费者用 `is_changed()` 订阅
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct AimFocus {
    pub hit: Option<TargetHit>,
    pub block: Option<AimBlockInfo>,
    pub structure_id: Option<StructureId>,
    /// 瞄准告示且有非空文本时的展示文案
    pub sign_label: Option<String>,
    /// 放置预览格：`hit.pos + hit.normal`（无瞄准时为 None）
    pub place_at: Option<IVec3>,
}

/// 在射线/手势更新 target 之后，解析一次瞄准派生上下文
pub fn sync_aim_focus(
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    structure_state: Res<StructureState>,
    mut aim: ResMut<AimFocus>,
) {
    let next = match placement.target {
        None => AimFocus::default(),
        Some(hit) => {
            let (block, sign_label) = resolve_aimed_block(&world, hit.pos);
            let structure_id = structure_state
                .structure_id_at(hit.pos)
                .filter(|id| !id.is_none());
            AimFocus {
                hit: Some(hit),
                block,
                structure_id,
                sign_label,
                place_at: Some(hit.pos + hit.normal),
            }
        }
    };
    if *aim != next {
        *aim = next;
    }
}

fn resolve_aimed_block(world: &WorldBlocks, pos: IVec3) -> (Option<AimBlockInfo>, Option<String>) {
    if let Some(block) = world.blocks.get(&pos) {
        let sign_label = (block.kind == BlockKind::Sign)
            .then(|| world.sign_settings(pos))
            .and_then(|settings| settings.text)
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty());
        return (
            Some(AimBlockInfo {
                kind: block.kind,
                facing: block.facing,
                id: block.id,
                system_layer: false,
            }),
            sign_label,
        );
    }
    if let Some(block) = world.system_blocks.get(&pos) {
        return (
            Some(AimBlockInfo {
                kind: block.kind,
                facing: block.facing,
                id: block.id,
                system_layer: true,
            }),
            None,
        );
    }
    (None, None)
}
