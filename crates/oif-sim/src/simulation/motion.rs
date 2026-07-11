use glam::IVec3;

use crate::blocks::BlockId;
use crate::world::direction::Facing;

/// 方块运动种类（纯数据，非 Bevy Component）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockMotionKind {
    Move,
    Rotate { pivot: IVec3, clockwise: bool },
    SpawnScale,
}

/// 单格方块本回合运动描述（TurnOutput DTO）
#[derive(Clone, Copy, Debug)]
pub struct BlockMotion {
    pub block_id: BlockId,
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub from_facing: Facing,
    pub to_facing: Facing,
    pub kind: BlockMotionKind,
}

/// 推杆伸出量本回合变化（TurnOutput DTO；duration 由放映层填写）
#[derive(Clone, Copy, Debug)]
pub struct PusherMotion {
    pub from_extension: f32,
    pub to_extension: f32,
}
