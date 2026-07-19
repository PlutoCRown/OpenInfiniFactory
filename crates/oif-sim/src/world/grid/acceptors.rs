//! 验收结构连通与材料验收匹配

use glam::IVec3;
use std::collections::HashSet;

use crate::blocks::{
    AcceptorId, BlockId, BlockKind, MaterialBlockId, PaintMaterialId, StampMaterialId,
};
use crate::world::direction::Facing;

use super::{
    BlockSettings, GeneratorMode, GeneratorSettings, GoalSettings, StoredAcceptorStructure,
    WorldBlocks,
};

const ACCEPTOR_NEIGHBOR_OFFSETS: [IVec3; 6] = [
    IVec3::X,
    IVec3::NEG_X,
    IVec3::Y,
    IVec3::NEG_Y,
    IVec3::Z,
    IVec3::NEG_Z,
];

impl WorldBlocks {
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
                            .is_some_and(|block| block.kind.accepts_material())
                    })
                    .map(|rep| (structure.id, rep))
            })
            .collect();

        let mut handled = HashSet::new();
        let mut starts: Vec<IVec3> = self
            .system_blocks
            .iter()
            .filter_map(|(pos, block)| block.kind.accepts_material().then_some(*pos))
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
                    .is_some_and(|block| block.kind.accepts_material())
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

    /// 验收：材料种类/朝向匹配，且印花与漆的附着多重集合精确匹配设定
    pub fn accepts_material_id_at(
        &self,
        pos: IVec3,
        material: MaterialBlockId,
        facing: Facing,
        block_id: BlockId,
    ) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| {
                if !block.kind.accepts_material() {
                    return false;
                }
                let settings = self.goal_settings(pos);
                if settings.material != material {
                    return false;
                }
                if BlockKind::Material(material).is_directional() && settings.facing != facing {
                    return false;
                }
                self.goal_attachments_match(block_id, &settings)
            })
    }

    /// 目标设定与方块实际印花/漆附着是否精确一致（空槽不计入要求）
    fn goal_attachments_match(&self, block_id: BlockId, settings: &GoalSettings) -> bool {
        let mut required_stamps: Vec<StampMaterialId> =
            settings.stamps.iter().copied().flatten().collect();
        required_stamps.sort_unstable();
        let mut actual_stamps: Vec<StampMaterialId> = self
            .material_attachments
            .iter()
            .filter(|(_, att)| att.parent == block_id)
            .filter_map(|(child_id, _)| {
                self.blocks
                    .values()
                    .find(|block| block.id == *child_id)
                    .and_then(|block| block.kind.stamp_id())
            })
            .collect();
        actual_stamps.sort_unstable();
        if actual_stamps != required_stamps {
            return false;
        }

        let mut required_paints: Vec<PaintMaterialId> =
            settings.paints.iter().copied().flatten().collect();
        required_paints.sort_unstable();
        let mut actual_paints: Vec<PaintMaterialId> = self
            .material_paints
            .iter()
            .filter(|(face, _)| face.block == block_id)
            .map(|(_, paint)| *paint)
            .collect();
        actual_paints.sort_unstable();
        actual_paints == required_paints
    }
}
