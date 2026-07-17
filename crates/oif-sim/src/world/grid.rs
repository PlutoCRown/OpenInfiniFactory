use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::blocks::{AcceptorId, BlockData, BlockId, BlockKind, MaterialKind, StampColor};
use crate::world::direction::Facing;

pub const REACH: f32 = 12.0;
pub const FLOOR_RADIUS: i32 = 12;
const TELEPORT_ENTRANCE_NAMES: &[&str] = &["Alpha In", "Beta In", "Gamma In", "Delta In"];
const TELEPORT_EXIT_NAMES: &[&str] = &["Alpha Out", "Beta Out", "Gamma Out", "Delta Out"];
const ACCEPTOR_NEIGHBOR_OFFSETS: [IVec3; 6] = [
    IVec3::X,
    IVec3::NEG_X,
    IVec3::Y,
    IVec3::NEG_Y,
    IVec3::Z,
    IVec3::NEG_Z,
];

#[derive(Default, Clone)]
pub struct WorldBlocks {
    pub blocks: HashMap<IVec3, BlockData>,
    pub system_blocks: HashMap<IVec3, BlockData>,
    pub material_welds: HashSet<MaterialWeld>,
    /// 材料面装饰漆：按 BlockId+法线键控，移动无需改写
    pub material_paints: HashMap<MaterialFace, StampColor>,
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
    /// 测试用：强制视为不可 Connectable 的材料面
    #[cfg(test)]
    pub test_unconnectable_faces: HashSet<MaterialFace>,
}

/// 编辑态派生的验收结构（无验收计数，不写入存档）
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredAcceptorStructure {
    pub id: AcceptorId,
    pub positions: Vec<IVec3>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockSettings {
    Generator(GeneratorSettings),
    Goal(GoalSettings),
    Labeler(LabelerSettings),
    Converter(ConverterSettings),
    Teleport(TeleportSettings),
}

impl BlockSettings {
    fn matches_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Generator(_), Self::Generator(_))
                | (Self::Goal(_), Self::Goal(_))
                | (Self::Labeler(_), Self::Labeler(_))
                | (Self::Converter(_), Self::Converter(_))
                | (Self::Teleport(_), Self::Teleport(_))
        )
    }
}

/// 生成器触发模式：周期或连接验收结构（Link 存代表 Goal 坐标，加载时由连通性解析）
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GeneratorMode {
    Period { period: u64, offset: u64 },
    Link { anchor: Option<IVec3> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    pub mode: GeneratorMode,
    pub material: MaterialKind,
}

impl GeneratorSettings {
    /// 同参相连判定键（忽略材料种类）
    pub fn trigger_key(self) -> GeneratorMode {
        match self.mode {
            GeneratorMode::Period { period, offset } => GeneratorMode::Period {
                period: period.max(1),
                offset: offset % period.max(1),
            },
            link => link,
        }
    }

    pub fn clamps_offset(mut self) -> Self {
        if let GeneratorMode::Period { period, offset } = &mut self.mode {
            let p = (*period).max(1);
            *period = p;
            *offset %= p;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GoalSettings {
    pub material: MaterialKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LabelerSettings {
    pub color: StampColor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConverterSettings {
    pub mode: ConverterMode,
    pub input: MaterialKind,
    pub output: MaterialKind,
}

impl Default for ConverterSettings {
    fn default() -> Self {
        Self {
            mode: ConverterMode::AnyInput,
            input: MaterialKind::Basic,
            output: MaterialKind::Iron,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConverterMode {
    AnyInput,
    SpecificInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeleportSettings {
    pub name: String,
    pub pair: Option<IVec3>,
}

impl TeleportSettings {
    pub fn unnamed(pos: IVec3) -> Self {
        Self {
            name: format!("Portal {}", pos_hash(pos)),
            pair: None,
        }
    }
}

impl Default for LabelerSettings {
    fn default() -> Self {
        Self {
            color: StampColor::Red,
        }
    }
}

impl Default for GoalSettings {
    fn default() -> Self {
        Self {
            material: MaterialKind::Basic,
        }
    }
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

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            mode: GeneratorMode::Period {
                period: crate::blocks::DEFAULT_GENERATOR_PERIOD,
                offset: 0,
            },
            material: MaterialKind::Basic,
        }
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
    // 仅为工厂/材料分配实例 ID；系统、虚拟、场景不参与动画体系
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
        if kind == BlockKind::Goal {
            self.resync_acceptor_structures();
        }
        previous
    }

    pub fn remove(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.blocks.remove(pos);
        if let Some(ref block) = removed {
            let id = block.id;
            if !id.is_none() {
                self.material_welds.retain(|weld| !weld.contains(id));
                self.material_paints.retain(|face, _| face.block != id);
                self.wire_face_panels.retain(|face| face.block != id);
            }
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        removed
    }

    pub fn remove_system(&mut self, pos: &IVec3) -> Option<BlockData> {
        let removed = self.system_blocks.remove(pos);
        if removed.is_some() {
            let was_goal = removed
                .as_ref()
                .is_some_and(|block| block.kind == BlockKind::Goal);
            self.block_settings.remove(pos);
            for settings in self.block_settings.values_mut() {
                if let BlockSettings::Teleport(settings) = settings {
                    if settings.pair == Some(*pos) {
                        settings.pair = None;
                    }
                }
            }
            self.topology_revision = self.topology_revision.wrapping_add(1);
            if was_goal {
                self.resync_acceptor_structures();
            }
        }
        removed
    }

    pub fn clear(&mut self) {
        if !self.blocks.is_empty()
            || !self.system_blocks.is_empty()
            || !self.acceptor_structures.is_empty()
            || !self.material_paints.is_empty()
            || !self.wire_face_panels.is_empty()
        {
            self.blocks.clear();
            self.system_blocks.clear();
            self.material_welds.clear();
            self.material_paints.clear();
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
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    /// 在电线指定面放置/移除灯面板；变更时 bump 信号拓扑
    pub fn set_wire_face_panel(&mut self, face: MaterialFace, present: bool) -> bool {
        let changed = if present {
            self.wire_face_panels.insert(face)
        } else {
            self.wire_face_panels.remove(&face)
        };
        if changed {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
        changed
    }

    /// 批量搬迁方块：保留 BlockId，不改焊接/面附着；信号相关方块移动时只 bump 一次 topology
    pub fn relocate_blocks(&mut self, moves: Vec<(IVec3, IVec3, BlockData)>) {
        if moves.is_empty() {
            return;
        }
        let touches_signals = moves
            .iter()
            .any(|(_, _, block)| block.kind.signal_behavior(block.facing).is_some());
        for (from, _, _) in &moves {
            self.blocks.remove(from);
        }
        for (_, to, block) in moves {
            self.blocks.insert(to, block);
        }
        if touches_signals {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn generator_settings(&self, pos: IVec3) -> GeneratorSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Generator(settings)) => *settings,
            _ => GeneratorSettings::default(),
        }
    }

    pub fn set_block_settings(&mut self, pos: IVec3, settings: BlockSettings) {
        let Some(block) = self.system_blocks.get(&pos).copied() else {
            return;
        };
        let Some(default_settings) = block.kind.default_settings(pos) else {
            return;
        };
        if !settings.matches_kind(&default_settings) {
            return;
        }
        if self.block_settings.get(&pos) != Some(&settings) {
            self.block_settings.insert(pos, settings);
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    pub fn set_generator_settings(&mut self, pos: IVec3, settings: GeneratorSettings) {
        self.set_block_settings(pos, BlockSettings::Generator(settings.clamps_offset()));
    }

    /// 查询 Goal 格所属验收结构 ID
    pub fn acceptor_id_at(&self, pos: IVec3) -> Option<AcceptorId> {
        self.acceptor_structures
            .iter()
            .find(|structure| structure.positions.iter().any(|p| *p == pos))
            .map(|structure| structure.id)
    }

    /// 按当前 Goal 连通重算验收结构，尽量保留代表格所在块的旧 ID
    pub fn resync_acceptor_structures(&mut self) {
        let old = std::mem::take(&mut self.acceptor_structures);
        let old_reps: Vec<(AcceptorId, IVec3)> = old
            .iter()
            .filter_map(|structure| {
                let mut positions = structure.positions.clone();
                positions.sort_by_key(|pos| (pos.x, pos.y, pos.z));
                positions
                    .into_iter()
                    .find(|pos| {
                        self.system_blocks
                            .get(pos)
                            .is_some_and(|block| block.kind == BlockKind::Goal)
                    })
                    .map(|rep| (structure.id, rep))
            })
            .collect();

        let mut handled = HashSet::new();
        let mut starts: Vec<IVec3> = self
            .system_blocks
            .iter()
            .filter_map(|(pos, block)| (block.kind == BlockKind::Goal).then_some(*pos))
            .collect();
        starts.sort_by_key(|pos| (pos.x, pos.y, pos.z));

        let mut next = Vec::new();
        let mut alive = HashSet::new();
        for start in starts {
            if handled.contains(&start) {
                continue;
            }
            let positions = self.connected_goal_positions(start);
            handled.extend(positions.iter().copied());

            let owners: Vec<AcceptorId> = old_reps
                .iter()
                .filter(|(_, rep)| positions.contains(rep))
                .map(|(id, _)| *id)
                .collect();
            let id = if let Some(id) = owners.into_iter().min() {
                id
            } else {
                self.next_acceptor_id = self.next_acceptor_id.max(1);
                let id = AcceptorId(self.next_acceptor_id);
                self.next_acceptor_id += 1;
                id
            };
            alive.insert(id);
            let mut sorted = positions;
            sorted.sort_by_key(|pos| (pos.x, pos.y, pos.z));
            next.push(StoredAcceptorStructure {
                id,
                positions: sorted,
            });
        }

        self.acceptor_structures = next;
        self.invalidate_stale_generator_links(&alive);
    }

    fn connected_goal_positions(&self, start: IVec3) -> Vec<IVec3> {
        let mut structure = Vec::new();
        let mut queue = std::collections::VecDeque::from([start]);
        let mut seen = HashSet::from([start]);
        while let Some(pos) = queue.pop_front() {
            structure.push(pos);
            for offset in ACCEPTOR_NEIGHBOR_OFFSETS {
                let neighbor = pos + offset;
                if seen.contains(&neighbor) {
                    continue;
                }
                if self
                    .system_blocks
                    .get(&neighbor)
                    .is_some_and(|block| block.kind == BlockKind::Goal)
                {
                    seen.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        structure
    }

    fn invalidate_stale_generator_links(&mut self, alive: &HashSet<AcceptorId>) {
        let stale: Vec<IVec3> = self
            .block_settings
            .iter()
            .filter_map(|(pos, settings)| match settings {
                BlockSettings::Generator(GeneratorSettings {
                    mode: GeneratorMode::Link { anchor },
                    ..
                }) => {
                    let broken = match anchor {
                        None => false,
                        Some(anchor) => self
                            .acceptor_id_at(*anchor)
                            .is_none_or(|id| !alive.contains(&id)),
                    };
                    broken.then_some(*pos)
                }
                _ => None,
            })
            .collect();
        for pos in stale {
            let mut settings = self.generator_settings(pos);
            settings.mode = GeneratorMode::Link { anchor: None };
            self.block_settings
                .insert(pos, BlockSettings::Generator(settings));
        }
    }

    pub fn goal_settings(&self, pos: IVec3) -> GoalSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Goal(settings)) => *settings,
            _ => GoalSettings::default(),
        }
    }

    pub fn set_goal_settings(&mut self, pos: IVec3, settings: GoalSettings) {
        self.set_block_settings(pos, BlockSettings::Goal(settings));
    }

    pub fn labeler_settings(&self, pos: IVec3) -> LabelerSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Labeler(settings)) => *settings,
            _ => LabelerSettings::default(),
        }
    }

    pub fn set_labeler_settings(&mut self, pos: IVec3, settings: LabelerSettings) {
        self.set_block_settings(pos, BlockSettings::Labeler(settings));
    }

    pub fn converter_settings(&self, pos: IVec3) -> ConverterSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Converter(settings)) => *settings,
            _ => ConverterSettings::default(),
        }
    }

    pub fn set_converter_settings(&mut self, pos: IVec3, settings: ConverterSettings) {
        self.set_block_settings(pos, BlockSettings::Converter(settings));
    }

    pub fn teleport_settings(&self, pos: IVec3) -> TeleportSettings {
        match self.block_settings.get(&pos) {
            Some(BlockSettings::Teleport(settings)) => settings.clone(),
            _ => TeleportSettings::unnamed(pos),
        }
    }

    pub fn teleport_partner(&self, pos: IVec3) -> Option<IVec3> {
        if let Some(pair) = self.teleport_settings(pos).pair {
            if self
                .system_blocks
                .get(&pair)
                .is_some_and(|block| self.teleport_kinds_match(pos, pair, block.kind))
            {
                return Some(pair);
            }
        }
        for (other_pos, settings) in &self.block_settings {
            if *other_pos == pos {
                continue;
            }
            let BlockSettings::Teleport(settings) = settings else {
                continue;
            };
            if settings.pair != Some(pos) {
                continue;
            }
            let Some(block) = self.system_blocks.get(other_pos) else {
                continue;
            };
            if self.teleport_kinds_match(pos, *other_pos, block.kind) {
                return Some(*other_pos);
            }
        }
        None
    }

    pub fn set_teleport_pair(&mut self, pos: IVec3, partner: Option<IVec3>) {
        let Some(block) = self.system_blocks.get(&pos).copied() else {
            return;
        };
        if !matches!(
            block.kind,
            BlockKind::TeleportEntrance | BlockKind::TeleportExit
        ) {
            return;
        }

        if let Some(old) = self.teleport_settings(pos).pair {
            if partner != Some(old) {
                let mut old_settings = self.teleport_settings(old);
                if old_settings.pair == Some(pos) {
                    old_settings.pair = None;
                    self.set_teleport_settings(old, old_settings);
                }
            }
        }

        if let Some(partner_pos) = partner {
            let Some(partner_block) = self.system_blocks.get(&partner_pos).copied() else {
                return;
            };
            if !self.teleport_kinds_match(pos, partner_pos, partner_block.kind) {
                return;
            }

            if let Some(previous) = self.teleport_settings(partner_pos).pair {
                if previous != pos {
                    let mut previous_settings = self.teleport_settings(previous);
                    previous_settings.pair = None;
                    self.set_teleport_settings(previous, previous_settings);
                }
            }

            let mut partner_settings = self.teleport_settings(partner_pos);
            partner_settings.pair = Some(pos);
            self.set_teleport_settings(partner_pos, partner_settings);
        }

        let mut settings = self.teleport_settings(pos);
        settings.pair = partner;
        self.set_teleport_settings(pos, settings);
    }

    pub fn set_teleport_settings(&mut self, pos: IVec3, settings: TeleportSettings) {
        self.set_block_settings(pos, BlockSettings::Teleport(settings));
    }

    fn teleport_kinds_match(&self, pos: IVec3, other: IVec3, other_kind: BlockKind) -> bool {
        let Some(block) = self.system_blocks.get(&pos) else {
            return false;
        };
        matches!(
            (block.kind, other_kind),
            (BlockKind::TeleportEntrance, BlockKind::TeleportExit)
                | (BlockKind::TeleportExit, BlockKind::TeleportEntrance)
        ) && pos != other
    }

    fn next_teleport_name(&self, kind: BlockKind) -> String {
        let base_names = match kind {
            BlockKind::TeleportEntrance => TELEPORT_ENTRANCE_NAMES,
            BlockKind::TeleportExit => TELEPORT_EXIT_NAMES,
            _ => &[],
        };
        let used: HashSet<String> = self
            .block_settings
            .iter()
            .filter_map(|(pos, settings)| {
                if !self
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| block.kind == kind)
                {
                    return None;
                }
                match settings {
                    BlockSettings::Teleport(settings) => Some(settings.name.clone()),
                    _ => None,
                }
            })
            .collect();

        for name in base_names {
            if !used.contains(*name) {
                return (*name).to_owned();
            }
        }

        for index in 2.. {
            for name in base_names {
                let candidate = format!("{name} {index}");
                if !used.contains(&candidate) {
                    return candidate;
                }
            }
        }
        unreachable!()
    }

    pub fn weld_materials(&mut self, a: IVec3, b: IVec3) -> bool {
        let Some(block_a) = self
            .blocks
            .get(&a)
            .copied()
            .filter(|block| block.kind.is_material() && !block.id.is_none())
        else {
            return false;
        };
        let Some(block_b) = self
            .blocks
            .get(&b)
            .copied()
            .filter(|block| block.kind.is_material() && !block.id.is_none())
        else {
            return false;
        };
        if block_a.id == block_b.id {
            return false;
        }
        let offset = b - a;
        if !block_a
            .kind
            .material_face_connectable(block_a.facing, offset)
            || !block_b
                .kind
                .material_face_connectable(block_b.facing, -offset)
        {
            return false;
        }
        if self.material_welds.insert(MaterialWeld::new(block_a.id, block_b.id)) {
            self.topology_revision = self.topology_revision.wrapping_add(1);
            true
        } else {
            false
        }
    }

    pub fn is_occupied(&self, pos: IVec3) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| block.kind.has_collision())
            || self
                .blocks
                .get(&pos)
                .is_some_and(|block| block.kind.has_collision())
    }

    pub fn is_platform_occupied(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.has_collision())
    }

    pub fn can_place_platform_at(&self, pos: IVec3) -> bool {
        !self.is_platform_occupied(pos)
    }

    pub fn has_system_block_at(&self, pos: IVec3) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_system_block())
    }

    pub fn has_generated_marker_at(&self, pos: IVec3) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_generated_marker())
            || self
                .blocks
                .get(&pos)
                .is_some_and(|block| block.kind.is_generated_marker())
    }

    pub fn blocks_factory_or_scene_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_factory() || block.kind.is_scene())
    }

    pub fn can_place_blocks_layer_at(&self, pos: IVec3, kind: BlockKind) -> bool {
        if pos.y < 0 {
            return false;
        }
        if self.is_platform_occupied(pos) {
            return false;
        }
        if kind.is_material() {
            return true;
        }
        if kind.is_factory() || kind.is_scene() {
            return !self.has_system_block_at(pos) && !self.has_generated_marker_at(pos);
        }
        false
    }

    pub fn can_place_system_block_at(&self, pos: IVec3) -> bool {
        if pos.y < 0 || self.has_system_block_at(pos) {
            return false;
        }
        !self.blocks_factory_or_scene_at(pos)
    }

    pub fn can_place_virtual_block_at(&self, pos: IVec3) -> bool {
        !self.system_blocks.contains_key(&pos)
    }

    pub fn can_place_block_kind_at(&self, pos: IVec3, kind: BlockKind) -> bool {
        if kind.is_system_block() {
            self.can_place_system_block_at(pos)
        } else if kind.is_generated_marker() {
            self.can_place_virtual_block_at(pos)
        } else {
            self.can_place_blocks_layer_at(pos, kind)
        }
    }

    pub fn can_move_into(&self, pos: IVec3) -> bool {
        !self.is_occupied(pos)
    }

    /// 格上是否为脆弱材料（运动冲突时让出并碎裂）
    pub fn is_fragile_material_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.material_props().is_some_and(|props| props.fragile))
    }

    /// 运动规划时该格是否可让出（空或脆弱材料）
    pub fn can_move_into_yielding_fragile(&self, pos: IVec3) -> bool {
        self.can_move_into(pos) || self.is_fragile_material_at(pos)
    }

    pub fn is_material_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_material())
    }

    pub fn is_teleport_entrance_at(&self, pos: IVec3) -> bool {
        self.system_blocks.get(&pos).is_some_and(|block| {
            matches!(
                block.kind.material_processor(),
                Some(crate::blocks::MaterialProcessor::TeleportEntrance)
            )
        })
    }

    pub fn anchors_material_at_teleport_entrance(&self, pos: IVec3) -> bool {
        self.is_material_at(pos) && self.is_teleport_entrance_at(pos)
    }

    pub fn is_factory_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_factory())
    }

    pub fn is_detectable_by_detector_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_detectable_by_detector())
    }

    pub fn is_scene_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_scene())
    }

    pub fn accepts_material_kind_at(&self, pos: IVec3, material: MaterialKind) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| match block.kind {
                BlockKind::Goal => self.goal_settings(pos).material == material,
                _ => block.kind.accepts_material(),
            })
    }

    pub fn clear_generated_markers(&mut self) {
        let blocks_before = self.blocks.len();
        self.blocks
            .retain(|_, block| !block.kind.is_generated_marker());
        let system_before = self.system_blocks.len();
        self.system_blocks
            .retain(|_, block| !block.kind.is_generated_marker());
        if self.blocks.len() != blocks_before || self.system_blocks.len() != system_before {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }
}

#[derive(Clone, Copy)]
pub struct TargetHit {
    pub pos: IVec3,
    pub normal: IVec3,
}

pub fn raycast_infinite_plane(
    origin: Vec3,
    dir: Vec3,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Option<Vec3> {
    let normal = plane_normal.normalize_or_zero();
    if normal == Vec3::ZERO {
        return None;
    }
    let denom = dir.dot(normal);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (plane_point - origin).dot(normal) / denom;
    if t < 0.0 || t > REACH {
        return None;
    }
    Some(origin + dir * t)
}

pub fn world_to_grid(pos: Vec3) -> IVec3 {
    pos.floor().as_ivec3()
}

/// 编辑框选模式（与配置层同构，避免 oif-sim 依赖主 crate）
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EditSelectionMode {
    #[default]
    Point,
    Line,
    Plane,
}

pub fn raycast_edit_drag_grid(
    origin: Vec3,
    dir: Vec3,
    start: IVec3,
    mode: EditSelectionMode,
    camera_dir: Vec3,
    plane_normal: IVec3,
) -> Option<IVec3> {
    if mode == EditSelectionMode::Point {
        return None;
    }

    let plane_point = grid_to_world(start);
    let plane_normal_vec = match mode {
        EditSelectionMode::Plane => plane_normal.as_vec3(),
        EditSelectionMode::Line => -camera_dir.normalize_or_zero(),
        EditSelectionMode::Point => unreachable!(),
    };
    if plane_normal_vec == Vec3::ZERO {
        return None;
    }

    let Some(hit) = raycast_infinite_plane(origin, dir, plane_point, plane_normal_vec) else {
        return None;
    };

    Some(match mode {
        EditSelectionMode::Plane => snap_plane_on_normal(hit, start, plane_normal),
        EditSelectionMode::Line => {
            let raw = world_to_grid(hit);
            let delta = raw - start;
            if delta == IVec3::ZERO {
                start
            } else {
                snap_line_on_plane(hit, start, strongest_axis_vec(delta))
            }
        }
        EditSelectionMode::Point => unreachable!(),
    })
}

fn snap_plane_on_normal(hit: Vec3, start: IVec3, normal: IVec3) -> IVec3 {
    let grid = world_to_grid(hit);
    if normal.x.abs() != 0 {
        IVec3::new(start.x, grid.y, grid.z)
    } else if normal.y.abs() != 0 {
        IVec3::new(grid.x, start.y, grid.z)
    } else {
        IVec3::new(grid.x, grid.y, start.z)
    }
}

fn strongest_axis_vec(delta: IVec3) -> IVec3 {
    if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        IVec3::X
    } else if delta.y.abs() >= delta.z.abs() {
        IVec3::Y
    } else {
        IVec3::Z
    }
}

fn snap_line_on_plane(hit: Vec3, start: IVec3, axis: IVec3) -> IVec3 {
    let grid = world_to_grid(hit);
    if axis.x != 0 {
        IVec3::new(grid.x, start.y, start.z)
    } else if axis.y != 0 {
        IVec3::new(start.x, grid.y, start.z)
    } else {
        IVec3::new(start.x, start.y, grid.z)
    }
}

pub fn seed_demo_world(world: &mut WorldBlocks) {
    for x in -FLOOR_RADIUS..=FLOOR_RADIUS {
        for z in -FLOOR_RADIUS..=FLOOR_RADIUS {
            world.insert(
                IVec3::new(x, 0, z),
                BlockData::new(BlockKind::Stone, Facing::North),
            );
        }
    }
}

pub fn grid_to_world(pos: IVec3) -> Vec3 {
    pos.as_vec3() + Vec3::splat(0.5)
}

fn pos_hash(pos: IVec3) -> i32 {
    pos.x.abs() * 31 + pos.y.abs() * 17 + pos.z.abs() * 13
}

pub fn raycast_blocks(origin: Vec3, dir: Vec3, world: &WorldBlocks) -> Option<TargetHit> {
    let dir = dir.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let mut best: Option<(f32, TargetHit)> = None;

    for (pos, block) in &world.blocks {
        if !block.kind.has_collision() || block.kind.is_generated_marker() {
            continue;
        }

        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max, REACH) {
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }
    for (pos, block) in &world.system_blocks {
        if block.kind.is_generated_marker() {
            continue;
        }
        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max, REACH) {
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }

    best.map(|(_, hit)| hit)
}

fn ray_aabb(
    origin: Vec3,
    dir: Vec3,
    min: Vec3,
    max: Vec3,
    max_distance: f32,
) -> Option<(f32, IVec3)> {
    let mut t_enter = 0.0;
    let mut t_exit = max_distance;
    let mut normal = IVec3::ZERO;

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let dir_axis = dir[axis];
        let min_axis = min[axis];
        let max_axis = max[axis];

        if dir_axis.abs() < 0.0001 {
            if origin_axis < min_axis || origin_axis > max_axis {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / dir_axis;
        let mut near = (min_axis - origin_axis) * inv_dir;
        let mut far = (max_axis - origin_axis) * inv_dir;
        let near_normal = if inv_dir >= 0.0 {
            -axis_vec(axis)
        } else {
            axis_vec(axis)
        };

        if near > far {
            std::mem::swap(&mut near, &mut far);
        }

        if near > t_enter {
            t_enter = near;
            normal = near_normal;
        }
        t_exit = t_exit.min(far);
        if t_enter > t_exit {
            return None;
        }
    }

    if t_exit < 0.0 {
        None
    } else {
        Some((t_enter.max(0.0), normal))
    }
}

fn axis_vec(axis: usize) -> IVec3 {
    match axis {
        0 => IVec3::X,
        1 => IVec3::Y,
        _ => IVec3::Z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POS: IVec3 = IVec3::new(1, 0, 2);

    #[test]
    fn factory_cannot_overlap_system_block() {
        let mut world = WorldBlocks::default();
        world.insert(POS, BlockData::new(BlockKind::Generator, Facing::North));

        assert!(!world.can_place_block_kind_at(POS, BlockKind::Platform));
        assert!(world.can_place_block_kind_at(POS, BlockKind::Material));
        assert!(world.can_place_platform_at(POS));
    }

    #[test]
    fn system_block_cannot_overlap_factory() {
        let mut world = WorldBlocks::default();
        world.insert(POS, BlockData::new(BlockKind::Platform, Facing::North));

        assert!(!world.can_place_block_kind_at(POS, BlockKind::Goal));
    }

    #[test]
    fn adjacent_goals_share_acceptor_id() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, BlockData::new(BlockKind::Goal, Facing::North));
        world.insert(IVec3::X, BlockData::new(BlockKind::Goal, Facing::North));
        world.insert(
            IVec3::new(0, 2, 0),
            BlockData::new(BlockKind::Goal, Facing::North),
        );

        assert_eq!(world.acceptor_structures.len(), 2);
        let id_a = world.acceptor_id_at(IVec3::ZERO).unwrap();
        let id_b = world.acceptor_id_at(IVec3::X).unwrap();
        let id_c = world.acceptor_id_at(IVec3::new(0, 2, 0)).unwrap();
        assert_eq!(id_a, id_b);
        assert_ne!(id_a, id_c);
        assert!(!id_a.is_none());
    }

    #[test]
    fn removing_goal_splits_acceptor_and_keeps_representative_id() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, BlockData::new(BlockKind::Goal, Facing::North));
        world.insert(IVec3::X, BlockData::new(BlockKind::Goal, Facing::North));
        world.insert(
            IVec3::new(2, 0, 0),
            BlockData::new(BlockKind::Goal, Facing::North),
        );
        let original = world.acceptor_id_at(IVec3::ZERO).unwrap();
        assert_eq!(world.acceptor_structures.len(), 1);

        world.remove_system(&IVec3::X);
        assert_eq!(world.acceptor_structures.len(), 2);
        assert_eq!(world.acceptor_id_at(IVec3::ZERO), Some(original));
        let other = world.acceptor_id_at(IVec3::new(2, 0, 0)).unwrap();
        assert_ne!(other, original);
    }

    #[test]
    fn factory_cannot_overlap_generated_marker() {
        let mut world = WorldBlocks::default();
        world.insert(POS, BlockData::new(BlockKind::DrillHead, Facing::North));

        assert!(!world.can_place_block_kind_at(POS, BlockKind::Wire));
        assert!(world.can_place_block_kind_at(POS, BlockKind::Material));
    }
}
