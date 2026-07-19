//! 模拟世界方块网格状态

mod acceptors;
mod attachments;
mod queries;
mod raycast;
mod settings;
mod teleport;

pub use raycast::{
    EditSelectionMode, TargetHit, grid_to_world, raycast_blocks, raycast_edit_drag_grid,
    raycast_infinite_plane, world_to_grid,
};
pub use settings::{
    BlockSettings, ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings, GoalSettings,
    RollerSettings, SignDisplay, SignSettings, StamperSettings, TeleportSettings,
};

use glam::IVec3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::blocks::{AcceptorId, BlockData, BlockId, PaintMaterialId};

/// 编辑瞄准最远距离（世界单位）
pub const REACH: f32 = 12.0;

/// 模拟世界方块网格：材料/系统/机身层与拓扑附属数据
#[derive(Default, Clone)]
pub struct WorldBlocks {
    pub blocks: HashMap<IVec3, BlockData>,
    pub system_blocks: HashMap<IVec3, BlockData>,
    /// 有碰撞机身占位（StamperBody/RollerBody）：与 System 宿主同格，不占 blocks 材料槽
    pub machine_bodies: HashMap<IVec3, BlockData>,
    pub material_welds: HashSet<MaterialWeld>,
    /// 材料面装饰漆：按 BlockId+法线键控，移动无需改写
    pub material_paints: HashMap<MaterialFace, PaintMaterialId>,
    /// 印花占格附着：子 BlockId → (父 BlockId, 父面法线)
    pub material_attachments: HashMap<BlockId, MaterialAttachment>,
    /// 告示等工厂占格附着：子工厂 BlockId → (父 BlockId, 父面法线)
    pub factory_attachments: HashMap<BlockId, MaterialAttachment>,
    /// 电线面灯面板：隔断该面信号连通，不占邻格
    pub wire_face_panels: HashSet<MaterialFace>,
    /// 系统层方块设置：按格子键控（系统块无 BlockId、不参与模拟移动）
    pub block_settings: HashMap<IVec3, BlockSettings>,
    /// 编辑态维护的验收结构（含持久 ID）
    pub acceptor_structures: Vec<StoredAcceptorStructure>,
    pub topology_revision: u64,
    /// 下一个可分配的方块实例 ID（0 表示未分配）
    pub next_block_id: u64,
    /// 下一个可分配的验收结构 ID
    pub next_acceptor_id: u64,
}

/// 印花等占格附着：子材料挂在父材料的某一面上
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MaterialAttachment {
    pub parent: BlockId,
    /// 从父指向子的世界法线（子占父+normal 格）
    pub parent_face_normal: IVec3,
}

/// 编辑态派生的验收结构（无验收计数，不写入存档）
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredAcceptorStructure {
    pub id: AcceptorId,
    pub positions: Vec<IVec3>,
}

/// 面附着键：按方块实例 ID + 世界法线（漆 / 灯面板等复用）
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MaterialFace {
    pub block: BlockId,
    pub normal: IVec3,
}

impl MaterialFace {
    pub fn new(block: BlockId, normal: IVec3) -> Self {
        Self { block, normal }
    }
}

/// 材料焊接：两端按 BlockId 排序存储，移动时无需改写
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MaterialWeld {
    pub a: BlockId,
    pub b: BlockId,
}

impl MaterialWeld {
    pub fn new(a: BlockId, b: BlockId) -> Self {
        if a.0 <= b.0 {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }

    pub fn other(self, id: BlockId) -> Option<BlockId> {
        if self.a == id {
            Some(self.b)
        } else if self.b == id {
            Some(self.a)
        } else {
            None
        }
    }

    pub fn contains(self, id: BlockId) -> bool {
        self.a == id || self.b == id
    }
}

impl WorldBlocks {
    pub fn assign_block_id(&mut self, block: &mut BlockData) {
        if !(block.kind.is_factory() || block.kind.is_material()) {
            block.id = crate::blocks::BlockId::NONE;
            return;
        }
        if block.id.is_none() {
            self.next_block_id = self.next_block_id.max(1);
            block.id = crate::blocks::BlockId(self.next_block_id);
            self.next_block_id += 1;
        } else {
            self.next_block_id = self.next_block_id.max(block.id.0.saturating_add(1));
        }
    }

    pub fn insert(&mut self, pos: IVec3, mut block: BlockData) -> Option<BlockData> {
        self.assign_block_id(&mut block);
        let kind = block.kind;
        let previous = if block.kind.is_system_layer() {
            self.system_blocks.insert(pos, block)
        } else {
            self.blocks.insert(pos, block)
        };
        if !self.block_settings.contains_key(&pos) {
            if let Some(mut settings) = kind.default_settings(pos) {
                if let BlockSettings::Teleport(teleport_settings) = &mut settings {
                    teleport_settings.name = self.next_teleport_name(kind);
                }
                self.block_settings.insert(pos, settings);
            }
        }
        if previous != Some(block) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        if kind.accepts_material() {
            self.resync_acceptor_structures();
        }
        previous
    }

    pub fn remove(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.blocks.remove(pos);
        if let Some(ref block) = removed {
            let id = block.id;
            // 材料可与系统块同格；有系统宿主时保留其 settings（如传送门配对）
            if !self.system_blocks.contains_key(pos) {
                self.block_settings.remove(pos);
            }
            if !id.is_none() {
                self.material_welds.retain(|weld| !weld.contains(id));
                self.material_paints.retain(|face, _| face.block != id);
                self.material_attachments.remove(&id);
                self.wire_face_panels.retain(|face| face.block != id);
                self.factory_attachments.remove(&id);
                // 宿主销毁时一并拆掉附着子块（印花材料 / 告示工厂）
                let material_children: Vec<BlockId> = self
                    .material_attachments
                    .iter()
                    .filter(|(_, att)| att.parent == id)
                    .map(|(child, _)| *child)
                    .collect();
                for child_id in material_children {
                    self.material_attachments.remove(&child_id);
                    self.material_paints
                        .retain(|face, _| face.block != child_id);
                    if let Some(child_pos) = self
                        .blocks
                        .iter()
                        .find(|(_, b)| b.id == child_id)
                        .map(|(p, _)| *p)
                    {
                        self.blocks.remove(&child_pos);
                        self.block_settings.remove(&child_pos);
                    }
                }
                let factory_children: Vec<BlockId> = self
                    .factory_attachments
                    .iter()
                    .filter(|(_, att)| att.parent == id)
                    .map(|(child, _)| *child)
                    .collect();
                for child_id in factory_children {
                    self.factory_attachments.remove(&child_id);
                    if let Some(child_pos) = self
                        .blocks
                        .iter()
                        .find(|(_, b)| b.id == child_id)
                        .map(|(p, _)| *p)
                    {
                        self.blocks.remove(&child_pos);
                        self.block_settings.remove(&child_pos);
                    }
                }
            }
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn remove_system(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.system_blocks.remove(pos);
        if removed.is_some() {
            let was_acceptor = removed
                .as_ref()
                .is_some_and(|block| block.kind.accepts_material());
            self.block_settings.remove(pos);
            for settings in self.block_settings.values_mut() {
                if let BlockSettings::Teleport(settings) = settings {
                    if settings.pair == Some(*pos) {
                        settings.pair = None;
                    }
                }
            }
            self.topology_revision = self.topology_revision.wrapping_add(1);
            if was_acceptor {
                self.resync_acceptor_structures();
            }
        }
        removed
    }

    pub fn clear(&mut self) {
        if !self.blocks.is_empty()
            || !self.system_blocks.is_empty()
            || !self.machine_bodies.is_empty()
            || !self.acceptor_structures.is_empty()
            || !self.material_paints.is_empty()
            || !self.material_attachments.is_empty()
            || !self.factory_attachments.is_empty()
            || !self.wire_face_panels.is_empty()
        {
            self.blocks.clear();
            self.system_blocks.clear();
            self.machine_bodies.clear();
            self.material_welds.clear();
            self.material_paints.clear();
            self.material_attachments.clear();
            self.factory_attachments.clear();
            self.wire_face_panels.clear();
            self.block_settings.clear();
            self.acceptor_structures.clear();
            self.next_acceptor_id = 0;
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&IVec3, &BlockData) -> bool) {
        let before = self.blocks.len();
        self.blocks.retain(|pos, block| keep(pos, block));
        if self.blocks.len() != before {
            let alive: HashSet<BlockId> = self.blocks.values().map(|block| block.id).collect();
            self.material_welds
                .retain(|weld| alive.contains(&weld.a) && alive.contains(&weld.b));
            self.material_paints
                .retain(|face, _| alive.contains(&face.block));
            self.wire_face_panels
                .retain(|face| alive.contains(&face.block));
            self.material_attachments
                .retain(|child, att| alive.contains(child) && alive.contains(&att.parent));
            self.factory_attachments
                .retain(|child, att| alive.contains(child) && alive.contains(&att.parent));
            self.block_settings.retain(|pos, _| {
                self.blocks.contains_key(pos) || self.system_blocks.contains_key(pos)
            });
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn clear_generated_markers(&mut self) {
        let blocks_before = self.blocks.len();
        self.blocks
            .retain(|_, block| !block.kind.is_generated_marker());
        let system_before = self.system_blocks.len();
        self.system_blocks
            .retain(|_, block| !block.kind.is_generated_marker());
        let bodies_before = self.machine_bodies.len();
        self.machine_bodies.clear();
        if self.blocks.len() != blocks_before
            || self.system_blocks.len() != system_before
            || bodies_before != 0
        {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }
}
