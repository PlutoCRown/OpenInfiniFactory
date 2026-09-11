//! 世界结构表：工厂/材料连通、可变形子集、回合 held

use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::blocks::{AcceptorId, BlockId, BlockKind, MovementRule};
use crate::world::grid::WorldBlocks;

use super::signal_offsets;

/// 结构运行时 ID：开局/焊接重建时分配，成员移动时保持不变
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct StructureId(pub u64);

impl StructureId {
    pub const NONE: Self = Self(0);

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoryActivity {
    Active,
    Inactive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureFreedom {
    None,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureKind {
    Material,
    Factory,
}

impl StructureFreedom {
    pub fn can_translate(self, _offset: IVec3) -> bool {
        self == Self::All
    }
}

/// 重力支撑接触：成员格 → 支撑方向
pub type GravitySupportContact = (IVec3, IVec3);

/// 共轴可变形子集：同节点集的正/反推动作必须一起伸缩
#[derive(Clone, Debug, Default)]
pub struct DeformGroup {
    pub actions: Vec<(BlockId, bool)>,
    pub nodes: Vec<BlockId>,
}

/// 单杆正/反推对应的格点两侧（由 DeformGroup 解析）
#[derive(Clone, Debug)]
pub struct DeformSides {
    pub separated: bool,
    pub actor_side: HashSet<IVec3>,
    pub target_side: HashSet<IVec3>,
    pub actor_anchored: bool,
    pub target_anchored: bool,
}

/// 结构成员的整数包围盒，由成员变更和位姿提交维护
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridBounds {
    pub min: IVec3,
    pub max: IVec3,
}

impl GridBounds {
    /// 从成员坐标构造包围盒，空结构没有包围盒
    pub fn from_positions(positions: &HashSet<IVec3>) -> Option<Self> {
        let mut positions = positions.iter().copied();
        let first = positions.next()?;
        let mut bounds = Self {
            min: first,
            max: first,
        };
        for pos in positions {
            bounds.min = bounds.min.min(pos);
            bounds.max = bounds.max.max(pos);
        }
        Some(bounds)
    }
}

/// 单个连通结构（工厂或材料）
#[derive(Clone)]
pub struct Structure {
    pub id: StructureId,
    pub kind: StructureKind,
    pub positions: HashSet<IVec3>,
    pub bounds: Option<GridBounds>,
    pub activity: FactoryActivity,
    pub freedom: StructureFreedom,
    gravity_support: Vec<GravitySupportContact>,
    /// 推杆体 → 逻辑头 BlockId（放置/重建时分配，伸出前后稳定）
    pub head_of: HashMap<BlockId, BlockId>,
    /// 逻辑头 → 推杆体
    body_of_head: HashMap<BlockId, BlockId>,
    pub deform_groups: Vec<DeformGroup>,
    /// (体, 正推?) → 候选组下标，按 nodes.len() 升序（优先多杆共轴、少节点）
    action_to_groups: HashMap<(BlockId, bool), Vec<u32>>,
    /// 开局/编辑时贴场景的成员格（锚死快照，中途落地不改）
    pub(crate) scene_touching: HashSet<IVec3>,
}

impl Structure {
    /// 是否可被推动：无贴场景成员
    pub fn is_pushable(&self) -> bool {
        self.scene_touching.is_empty()
    }

    /// 子集是否含贴场景成员（锚死不可推）
    pub fn is_scene_anchored_subset(&self, positions: &HashSet<IVec3>) -> bool {
        positions.iter().any(|p| self.scene_touching.contains(p))
    }
}

/// 验收口结构运行时计数
#[derive(Clone, Debug)]
pub struct AcceptorStructure {
    pub id: AcceptorId,
    pub positions: HashSet<IVec3>,
    pub count: u32,
}

/// 世界结构表：工厂/材料连通、可变形子集、回合 held
#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct StructureState {
    /// 已同步的材料身份；纯位姿提交保留成员历史，拓扑变化才重建
    pub(crate) material_topology: Option<std::sync::Arc<()>>,
    structures: HashMap<StructureId, Structure>,
    structure_by_pos: HashMap<IVec3, StructureId>,
    next_structure_id: u64,
    next_head_id: u64,
    acceptor_structures: Vec<AcceptorStructure>,
    /// 本回合已占用移动的方块（含逻辑头）；每回合清空
    pub held_blocks: HashSet<BlockId>,
    /// 本回合已占用整段平移的结构
    pub moving_structures: HashSet<StructureId>,
}

mod connectivity;
mod deform;
mod query;
mod rebuild;
use connectivity::bpos_facing_toward;
use connectivity::collect_gravity_support;
use connectivity::factory_structure;
use connectivity::is_blocked_factory_connection;
use connectivity::material_id_to_pos;
pub use connectivity::material_structure;
use connectivity::material_structure_from;
use connectivity::material_weld_neighbors;
pub use connectivity::query_factory_structure;
pub use connectivity::touches_scene;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
