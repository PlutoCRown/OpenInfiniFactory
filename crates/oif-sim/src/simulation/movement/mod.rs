//! 活塞状态与设备运动标记（传送带/推杆/抬升/旋转）

use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockData, BlockId, BlockKind, MovementRule};
use crate::world::grid::WorldBlocks;

use super::motion::PusherMotion;
use super::structure_state::{FactoryActivity, StructureId, StructureKind, StructureState};
use super::structures::{
    MovementMark, PusherActor, PusherAnimationKind, StructureMove, can_translate_structure,
};
use super::suction::SuctionLinks;

/// 活塞/拦截器工作面推/拉失败时是否反推自身（图预拆仍保留，仅跳过反推尝试）
pub const PUSHER_REVERSE_ENABLED: bool = true;

/// 推杆 deform 标记阶段的共享可变上下文
pub(super) struct SimMovementMarkCtx<'a> {
    pub world: &'a mut WorldBlocks,
    pub structures: &'a mut StructureState,
    pub suction: &'a SuctionLinks,
    pub claimed_heads: &'a mut HashSet<IVec3>,
    pub motion_held: &'a mut HashSet<IVec3>,
    pub motion_tags: &'a mut HashMap<IVec3, IVec3>,
    pub succeeded_deform: &'a mut HashSet<(StructureId, u32)>,
    pub actuating_extend: &'a HashSet<BlockId>,
    pub actuating_retract: &'a HashSet<BlockId>,
}

include!("pusher_state.rs");
include!("mark.rs");

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
