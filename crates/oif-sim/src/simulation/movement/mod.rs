//! 活塞状态与设备运动标记（传送带/推杆/抬升/旋转）

use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockData, BlockId, BlockKind, MovementRule};
use crate::world::grid::WorldBlocks;

use super::motion::PusherMotion;
use super::structure_state::{StructureId, StructureKind, StructureState};
use super::structures::{
    MovementMark, PusherActor, PusherAnimationKind, StructureMove, can_translate_structure,
};
use super::suction::SuctionLinks;

/// 活塞/拦截器工作面推/拉失败时是否反推自身（图预拆仍保留，仅跳过反推尝试）
pub const PUSHER_REVERSE_ENABLED: bool = true;

include!("pusher_state.rs");
include!("mark.rs");

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
