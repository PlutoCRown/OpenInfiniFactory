use std::collections::{HashMap, HashSet};

use glam::IVec3;

use crate::blocks::BlockKind;
use crate::world::grid::WorldBlocks;

use super::structure_state::{StructureId, StructureKind, StructureState};

/// 本回合通电吸盘建立的结构临时粘连（不合并 StructureId）
#[derive(Default, Clone)]
pub struct SuctionLinks {
    parent: HashMap<StructureId, StructureId>,
}

impl SuctionLinks {
    /// 根据通电吸盘重建粘连：面前有工厂/材料结构则与吸盘结构连通
    pub fn rebuild(
        world: &WorldBlocks,
        structures: &StructureState,
        powered_devices: &HashSet<IVec3>,
    ) -> Self {
        let mut links = Self::default();
        for &pos in powered_devices {
            let Some(block) = world.blocks.get(&pos) else {
                continue;
            };
            if block.kind != BlockKind::SuctionCup {
                continue;
            }
            let Some(self_id) = structures.id_at(pos) else {
                continue;
            };
            let front = pos + block.facing.forward_ivec3();
            let Some(front_block) = world.blocks.get(&front) else {
                continue;
            };
            if !front_block.kind.is_factory() && !front_block.kind.is_material() {
                continue;
            }
            let Some(front_id) = structures.id_at(front) else {
                continue;
            };
            if self_id != front_id {
                links.union(self_id, front_id);
            }
        }
        links
    }

    fn find(&self, id: StructureId) -> StructureId {
        let mut current = id;
        while let Some(&parent) = self.parent.get(&current) {
            if parent == current {
                break;
            }
            current = parent;
        }
        current
    }

    fn union(&mut self, a: StructureId, b: StructureId) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent.insert(rb, ra);
            self.parent.entry(ra).or_insert(ra);
        }
    }

    /// 与 seed 同一粘连分量的全部 StructureId
    pub fn component_ids(
        &self,
        structures: &StructureState,
        seed_ids: impl IntoIterator<Item = StructureId>,
    ) -> HashSet<StructureId> {
        let roots: HashSet<StructureId> = seed_ids.into_iter().map(|id| self.find(id)).collect();
        if roots.is_empty() {
            return HashSet::new();
        }
        let mut component = HashSet::new();
        for id in structures.structure_ids() {
            if roots.contains(&self.find(id)) {
                component.insert(id);
            }
        }
        // 无粘连记录时，find 等于自身，仍应包含 seed
        if component.is_empty() {
            for root in roots {
                component.insert(root);
            }
        }
        component
    }
}

impl StructureState {
    /// 经吸盘粘连扩展后的可平移位置并集；分量内任一结构不可动则 None
    pub fn linked_pushable_at(
        &self,
        links: &SuctionLinks,
        pos: IVec3,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        let seed_id = self.id_at(pos)?;
        self.linked_pushable_ids(links, [seed_id], offset)
    }

    fn linked_pushable_ids(
        &self,
        links: &SuctionLinks,
        seed_ids: impl IntoIterator<Item = StructureId>,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        let component = links.component_ids(self, seed_ids);
        let mut positions = HashSet::new();
        for id in component {
            let structure = self.structure_by_id(id)?;
            if !structure.pushable || !structure.freedom.can_translate(offset) {
                return None;
            }
            positions.extend(structure.positions.iter().copied());
        }
        (!positions.is_empty()).then_some(positions)
    }

    /// 活塞可推子集经吸盘扩展：子集本身不膨胀，只并入其它粘连结构的完整位置
    pub fn linked_expand_pusher_subset(
        &self,
        links: &SuctionLinks,
        subset: &HashSet<IVec3>,
        offset: IVec3,
    ) -> Option<HashSet<IVec3>> {
        let subset_ids: HashSet<StructureId> =
            subset.iter().filter_map(|pos| self.id_at(*pos)).collect();
        if subset_ids.is_empty() {
            return None;
        }
        let component = links.component_ids(self, subset_ids.iter().copied());
        let mut positions = subset.clone();
        for id in component {
            let structure = self.structure_by_id(id)?;
            if subset_ids.contains(&id) {
                // 种子子集已由调用方验证（如活塞子集），不因整结构 scene-anchored 而拒绝
                continue;
            }
            if !structure.pushable || !structure.freedom.can_translate(offset) {
                return None;
            }
            positions.extend(structure.positions.iter().copied());
        }
        Some(positions)
    }

    /// 粘连分量是否含工厂结构（旋转器遇此则失败）
    pub fn linked_contains_factory(
        &self,
        links: &SuctionLinks,
        seed_pos: IVec3,
    ) -> bool {
        let Some(seed_id) = self.id_at(seed_pos) else {
            return false;
        };
        links
            .component_ids(self, [seed_id])
            .into_iter()
            .any(|id| {
                self.structure_by_id(id)
                    .is_some_and(|s| s.kind == StructureKind::Factory)
            })
    }
}

