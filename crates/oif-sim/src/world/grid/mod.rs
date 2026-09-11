//! 模拟世界方块网格状态

mod acceptors;
mod attachments;
mod queries;
mod raycast;
mod settings;
mod teleport;

pub use raycast::{
    EditSelectionMode, FAR_FACE_PRIORITY_PENALTY, TargetHit, grid_to_world, raycast_blocks,
    raycast_edit_drag_grid, raycast_infinite_plane, world_to_grid,
};
pub use settings::{
    BlockSettings, ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings,
    GoalSettings, RollerSettings, SignDisplay, SignSettings, StamperSettings, TeleportSettings,
};

use glam::IVec3;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::blocks::{AcceptorId, BlockData, BlockId, BlockKind, PaintMaterialId};

/// 编辑瞄准最远距离（世界单位）
pub const REACH: f32 = 12.0;

/// 模拟世界方块网格：材料/系统层与拓扑附属数据
#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct WorldBlocks {
    pub(crate) blocks: HashMap<IVec3, BlockData>,
    pub(crate) system_blocks: HashMap<IVec3, BlockData>,
    pub(crate) material_welds: HashSet<MaterialWeld>,
    /// 材料面装饰漆：按 BlockId+法线键控，移动无需改写
    pub(crate) material_paints: HashMap<MaterialFace, PaintMaterialId>,
    /// 印花面附着：父材料面 → 印花材料，不占用网格格子
    pub(crate) material_stamps: HashMap<MaterialFace, crate::blocks::StampMaterialId>,
    /// 告示等工厂占格附着：子工厂 BlockId → (父 BlockId, 父面法线)
    pub(crate) factory_attachments: HashMap<BlockId, MaterialAttachment>,
    /// 电线面灯面板：隔断该面信号连通，不占邻格
    pub(crate) wire_face_panels: HashSet<MaterialFace>,
    /// 系统层方块设置：按格子键控（系统块无 BlockId、不参与模拟移动）
    pub(crate) block_settings: HashMap<IVec3, BlockSettings>,
    /// 编辑态维护的验收结构（含持久 ID）
    pub(crate) acceptor_structures: Vec<StoredAcceptorStructure>,
    pub(crate) topology_revision: u64,
    /// 信号接线身份：快照共享，接线变化后分离，避免跨世界版本号碰撞
    pub(crate) signal_topology: Arc<()>,
    /// 材料成员与焊接变化身份；结构提交位姿时同步消费这次变化
    pub(crate) material_topology: Arc<()>,
    /// 下一个可分配的方块实例 ID（0 表示未分配）
    pub(crate) next_block_id: u64,
    /// 下一个可分配的验收结构 ID
    pub(crate) next_acceptor_id: u64,
    /// 刚传送到达某格的材料 BlockId：同 id 留着则不传；空/换 id 则清标记再尝试
    pub(crate) teleport_arrivals: HashMap<IVec3, BlockId>,
    /// 旋转器工作面刚转过的材料 BlockId：同 id 留着则不转；空/换 id 则清；通电也可清
    pub(crate) rotator_arrivals: HashMap<IVec3, BlockId>,
    /// blocks 层场景方块数量（随 insert/remove 增量维护，供 UI/调试 O(1) 读取）
    pub(crate) scene_count: usize,
    /// blocks 层工厂方块数量
    pub(crate) factory_count: usize,
    /// blocks 层材料方块数量
    pub(crate) material_count: usize,
    /// 会生成静态 marker 的方块数量（包含 blocks 与 system_blocks）
    pub(crate) marker_source_count: usize,
    /// 当前静态生成 marker 数量（包含 blocks 与 system_blocks）
    pub(crate) generated_marker_count: usize,
}

/// 告示等占格附着：子工厂挂在父方块的某一面上
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
    /// 返回主方块层的只读映射。
    pub fn blocks(&self) -> &HashMap<IVec3, BlockData> {
        &self.blocks
    }

    /// 返回系统方块层的只读映射。
    pub fn system_blocks(&self) -> &HashMap<IVec3, BlockData> {
        &self.system_blocks
    }

    /// 返回材料焊缝的只读集合。
    pub fn material_welds(&self) -> &HashSet<MaterialWeld> {
        &self.material_welds
    }

    /// 返回材料面涂漆的只读映射。
    pub fn material_paints(&self) -> &HashMap<MaterialFace, PaintMaterialId> {
        &self.material_paints
    }

    /// 返回材料面印花的只读映射。
    pub fn material_stamps(&self) -> &HashMap<MaterialFace, crate::blocks::StampMaterialId> {
        &self.material_stamps
    }

    /// 返回工厂附着关系的只读映射。
    pub fn factory_attachments(&self) -> &HashMap<BlockId, MaterialAttachment> {
        &self.factory_attachments
    }

    /// 返回电线隔断面板的只读集合。
    pub fn wire_face_panels(&self) -> &HashSet<MaterialFace> {
        &self.wire_face_panels
    }

    /// 返回方块设置的只读映射。
    pub fn block_settings(&self) -> &HashMap<IVec3, BlockSettings> {
        &self.block_settings
    }

    /// 返回编辑态验收结构的只读切片。
    pub fn stored_acceptor_structures(&self) -> &[StoredAcceptorStructure] {
        &self.acceptor_structures
    }

    /// 当前世界拓扑版本，仅用于视图缓存失效判断
    pub fn topology_revision(&self) -> u64 {
        self.topology_revision
    }

    /// 场景、工厂和材料方块的增量计数
    pub fn block_counts(&self) -> (usize, usize, usize) {
        (self.scene_count, self.factory_count, self.material_count)
    }

    /// 接线变更时更换身份，供信号索引判断是否需要重建
    pub(crate) fn invalidate_signal_topology(&mut self) {
        self.signal_topology = Arc::new(());
    }

    /// 材料成员或焊接改变后，使结构派生数据失效
    pub(crate) fn invalidate_material_topology(&mut self) {
        self.material_topology = Arc::new(());
    }

    pub(crate) fn assign_block_id(&mut self, block: &mut BlockData) {
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

    /// 按方块类型增减 blocks 层分层计数（系统层不计入）
    fn adjust_block_count(&mut self, kind: BlockKind, delta: i32) {
        let slot = if kind.is_scene() {
            &mut self.scene_count
        } else if kind.is_factory() {
            &mut self.factory_count
        } else if kind.is_material() {
            &mut self.material_count
        } else {
            return;
        };
        if delta >= 0 {
            *slot = slot.saturating_add(delta as usize);
        } else {
            *slot = slot.saturating_sub((-delta) as usize);
        }
    }

    /// 增减静态 marker 来源与生成物计数，避免空场景反复全表扫描
    fn adjust_marker_count(&mut self, block: BlockData, delta: i32) {
        let amount = delta.unsigned_abs() as usize;
        if block.kind.marker_behavior(block.facing).is_some() {
            if delta >= 0 {
                self.marker_source_count = self.marker_source_count.saturating_add(amount);
            } else {
                self.marker_source_count = self.marker_source_count.saturating_sub(amount);
            }
        }
        if block.kind.is_generated_marker() {
            if delta >= 0 {
                self.generated_marker_count = self.generated_marker_count.saturating_add(amount);
            } else {
                self.generated_marker_count = self.generated_marker_count.saturating_sub(amount);
            }
        }
    }

    /// 从 blocks 全表重建分层计数（retain 等批量路径用）
    pub(crate) fn recount_block_counts(&mut self) {
        let mut scene = 0usize;
        let mut factory = 0usize;
        let mut material = 0usize;
        let mut marker_sources = 0usize;
        let mut generated_markers = 0usize;
        for block in self.blocks.values() {
            if block.kind.is_scene() {
                scene += 1;
            } else if block.kind.is_factory() {
                factory += 1;
            } else if block.kind.is_material() {
                material += 1;
            }
            marker_sources += usize::from(block.kind.marker_behavior(block.facing).is_some());
            generated_markers += usize::from(block.kind.is_generated_marker());
        }
        for block in self.system_blocks.values() {
            marker_sources += usize::from(block.kind.marker_behavior(block.facing).is_some());
            generated_markers += usize::from(block.kind.is_generated_marker());
        }
        self.scene_count = scene;
        self.factory_count = factory;
        self.material_count = material;
        self.marker_source_count = marker_sources;
        self.generated_marker_count = generated_markers;
    }

    pub fn insert(&mut self, pos: IVec3, mut block: BlockData) -> Option<BlockData> {
        self.assign_block_id(&mut block);
        let kind = block.kind;
        let previous = if block.kind.is_system_layer() {
            self.system_blocks.insert(pos, block)
        } else {
            self.blocks.insert(pos, block)
        };
        if let Some(ref prev) = previous {
            if !prev.kind.is_system_layer() {
                self.adjust_block_count(prev.kind, -1);
            }
            self.adjust_marker_count(*prev, -1);
        }
        if !kind.is_system_layer() {
            self.adjust_block_count(kind, 1);
        }
        self.adjust_marker_count(block, 1);
        if !self.block_settings.contains_key(&pos) {
            if let Some(mut settings) = kind.default_settings(pos) {
                if let BlockSettings::Teleport(teleport_settings) = &mut settings {
                    teleport_settings.name = self.next_teleport_name();
                }
                self.block_settings.insert(pos, settings);
            }
        }
        if previous != Some(block) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
            if block.kind.is_material() || previous.is_some_and(|old| old.kind.is_material()) {
                self.invalidate_material_topology();
            }
            if block.kind.signal_behavior(block.facing).is_some()
                || previous.is_some_and(|old| old.kind.signal_behavior(old.facing).is_some())
            {
                self.invalidate_signal_topology();
            }
        }
        if kind.accepts_material() {
            self.resync_acceptor_structures();
        }
        previous
    }

    pub fn remove(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.blocks.remove(pos);
        if let Some(ref block) = removed {
            if block.kind.is_material() {
                self.invalidate_material_topology();
            }
            if block.kind.signal_behavior(block.facing).is_some() {
                self.invalidate_signal_topology();
            }
            self.adjust_block_count(block.kind, -1);
            self.adjust_marker_count(*block, -1);
            let id = block.id;
            // 材料可与系统块同格；有系统宿主时保留其 settings（如传送门配对）
            if !self.system_blocks.contains_key(pos) {
                self.block_settings.remove(pos);
            }
            if !id.is_none() {
                self.material_welds.retain(|weld| !weld.contains(id));
                self.material_paints.retain(|face, _| face.block != id);
                self.material_stamps.retain(|face, _| face.block != id);
                self.wire_face_panels.retain(|face| face.block != id);
                self.factory_attachments.remove(&id);
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
                        if let Some(child) = self.blocks.remove(&child_pos) {
                            self.adjust_block_count(child.kind, -1);
                            self.adjust_marker_count(child, -1);
                            self.block_settings.remove(&child_pos);
                        }
                    }
                }
            }
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn remove_system(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.system_blocks.remove(pos);
        if let Some(block) = removed {
            self.adjust_marker_count(block, -1);
            let was_acceptor = block.kind.accepts_material();
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
            return Some(block);
        }
        None
    }

    /// 批量编辑与撤销按层恢复单格，并统一维护附着、计数、设置和拓扑身份
    pub fn restore_cell(
        &mut self,
        pos: IVec3,
        system_layer: bool,
        block: Option<BlockData>,
        settings: Option<BlockSettings>,
    ) {
        if system_layer {
            self.remove_system(&pos);
        } else {
            self.remove(&pos);
        }
        let Some(block) = block else {
            return;
        };
        debug_assert_eq!(system_layer, block.kind.is_system_layer());
        self.insert(pos, block);
        match settings {
            Some(settings) => self.set_block_settings(pos, settings),
            None => {
                if block.kind.default_settings(pos).is_none() {
                    self.block_settings.remove(&pos);
                }
            }
        }
    }

    /// 撤销与批量编辑恢复设置，并统一推进世界拓扑版本
    pub fn restore_block_settings(&mut self, pos: IVec3, settings: Option<BlockSettings>) {
        match settings {
            Some(settings) => {
                self.block_settings.insert(pos, settings);
            }
            None => {
                self.block_settings.remove(&pos);
            }
        }
        self.topology_revision = self.topology_revision.wrapping_add(1);
    }

    /// 批量应用材料焊缝差异并统一使材料结构缓存失效
    pub fn apply_material_weld_changes(
        &mut self,
        add: impl IntoIterator<Item = MaterialWeld>,
        remove: impl IntoIterator<Item = MaterialWeld>,
    ) {
        let mut changed = false;
        for weld in remove {
            changed |= self.material_welds.remove(&weld);
        }
        for weld in add {
            changed |= self.material_welds.insert(weld);
        }
        if changed {
            self.invalidate_material_topology();
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn clear(&mut self) {
        self.invalidate_signal_topology();
        self.invalidate_material_topology();
        if !self.blocks.is_empty()
            || !self.system_blocks.is_empty()
            || !self.acceptor_structures.is_empty()
            || !self.material_paints.is_empty()
            || !self.material_stamps.is_empty()
            || !self.factory_attachments.is_empty()
            || !self.wire_face_panels.is_empty()
            || !self.teleport_arrivals.is_empty()
            || !self.rotator_arrivals.is_empty()
        {
            self.blocks.clear();
            self.system_blocks.clear();
            self.material_welds.clear();
            self.material_paints.clear();
            self.material_stamps.clear();
            self.factory_attachments.clear();
            self.wire_face_panels.clear();
            self.block_settings.clear();
            self.acceptor_structures.clear();
            self.teleport_arrivals.clear();
            self.rotator_arrivals.clear();
            self.next_acceptor_id = 0;
            self.scene_count = 0;
            self.factory_count = 0;
            self.material_count = 0;
            self.marker_source_count = 0;
            self.generated_marker_count = 0;
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&IVec3, &BlockData) -> bool) {
        let before = self.blocks.len();
        let mut removed_signal = false;
        let mut removed_material = false;
        self.blocks.retain(|pos, block| {
            let retained = keep(pos, block);
            removed_material |= !retained && block.kind.is_material();
            removed_signal |= !retained && block.kind.signal_behavior(block.facing).is_some();
            retained
        });
        if removed_signal {
            self.invalidate_signal_topology();
        }
        if removed_material {
            self.invalidate_material_topology();
        }
        if self.blocks.len() != before {
            let alive: HashSet<BlockId> = self.blocks.values().map(|block| block.id).collect();
            self.material_welds
                .retain(|weld| alive.contains(&weld.a) && alive.contains(&weld.b));
            self.material_paints
                .retain(|face, _| alive.contains(&face.block));
            self.wire_face_panels
                .retain(|face| alive.contains(&face.block));
            self.material_stamps
                .retain(|face, _| alive.contains(&face.block));
            self.factory_attachments
                .retain(|child, att| alive.contains(child) && alive.contains(&att.parent));
            self.block_settings.retain(|pos, _| {
                self.blocks.contains_key(pos) || self.system_blocks.contains_key(pos)
            });
            self.recount_block_counts();
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
        if self.blocks.len() != blocks_before || self.system_blocks.len() != system_before {
            if self.blocks.len() != blocks_before {
                self.recount_block_counts();
            }
            self.generated_marker_count = 0;
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{MaterialBlockId, PaintMaterialId, SceneBlockId};
    use crate::world::Facing;

    /// 单格恢复统一清理材料附着与焊缝，并保持派生计数正确
    #[test]
    fn restore_cell_preserves_world_invariants() {
        let mut world = WorldBlocks::default();
        let pos = IVec3::new(1, 2, 3);
        let neighbor = pos + IVec3::X;
        world.insert(
            pos,
            BlockData::new(BlockKind::Material(MaterialBlockId(0)), Facing::North),
        );
        world.insert(
            neighbor,
            BlockData::new(BlockKind::Material(MaterialBlockId(0)), Facing::North),
        );
        let id = world.blocks[&pos].id;
        let neighbor_id = world.blocks[&neighbor].id;
        world
            .material_welds
            .insert(MaterialWeld::new(id, neighbor_id));
        world
            .material_paints
            .insert(MaterialFace::new(id, IVec3::Y), PaintMaterialId(0));

        world.restore_cell(
            pos,
            false,
            Some(BlockData::new(
                BlockKind::Scene(SceneBlockId(0)),
                Facing::North,
            )),
            None,
        );

        assert_eq!(world.material_count, 1);
        assert_eq!(world.scene_count, 1);
        assert!(world.material_welds.is_empty());
        assert!(world.material_paints.is_empty());
        assert!(world.blocks[&pos].kind.is_scene());
    }

    /// 分层恢复只替换目标层，保留同格的另一层方块
    #[test]
    fn restore_cell_preserves_other_layer() {
        let mut world = WorldBlocks::default();
        let pos = IVec3::new(1, 2, 3);
        let material = BlockData::new(BlockKind::Material(MaterialBlockId(0)), Facing::North);
        let system = BlockData::new(BlockKind::Goal, Facing::North);
        world.insert(pos, material);
        world.insert(pos, system);

        world.restore_cell(pos, true, None, None);

        assert_eq!(
            world.blocks.get(&pos).map(|block| block.kind),
            Some(material.kind)
        );
        assert!(!world.system_blocks.contains_key(&pos));
        assert_eq!(world.material_count, 1);
    }

    /// 清空世界同时清除只服务于当前模拟世界的到达标记
    #[test]
    fn clear_removes_arrival_markers_from_empty_grid() {
        let mut world = WorldBlocks::default();
        world.teleport_arrivals.insert(IVec3::ZERO, BlockId(1));
        world.rotator_arrivals.insert(IVec3::X, BlockId(2));

        world.clear();

        assert!(world.teleport_arrivals.is_empty());
        assert!(world.rotator_arrivals.is_empty());
    }
}
