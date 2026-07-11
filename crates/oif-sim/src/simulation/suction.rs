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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind};
    use crate::simulation::core::simulate_turn;
    use crate::simulation::movement::PusherState;
    use crate::simulation::pending::PendingGeneratedMaterials;
    use crate::simulation::signals::SignalNetworkCache;
    use crate::simulation::structures::MovementInfluenceCache;
    use crate::world::direction::Facing;

    fn place(world: &mut WorldBlocks, pos: IVec3, kind: BlockKind, facing: Facing) {
        world.insert(pos, BlockData::new(kind, facing));
    }

    fn sim_setup(world: WorldBlocks) -> (
        WorldBlocks,
        PendingGeneratedMaterials,
        SignalNetworkCache,
        StructureState,
        MovementInfluenceCache,
        PusherState,
    ) {
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        let pushers = PusherState::rebuild_from_world(&world);
        (
            world,
            PendingGeneratedMaterials::default(),
            SignalNetworkCache::default(),
            structures,
            MovementInfluenceCache::default(),
            pushers,
        )
    }

    /// 固定吸盘通电吸附材料 → 材料不下落
    #[test]
    fn fixed_suction_holds_material_from_gravity() {
        let mut world = WorldBlocks::default();
        // 地面锚定
        place(&mut world, IVec3::new(0, 0, 0), BlockKind::Stone, Facing::North);
        place(&mut world, IVec3::new(1, 0, 0), BlockKind::Stone, Facing::North);
        place(&mut world, IVec3::new(2, 0, 0), BlockKind::Stone, Facing::North);
        // 固定工厂：平台 + 朝东吸盘
        place(
            &mut world,
            IVec3::new(0, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(1, 1, 0),
            BlockKind::SuctionCup,
            Facing::East,
        );
        // 面前材料（悬空，若无吸附会下落）
        place(
            &mut world,
            IVec3::new(2, 1, 0),
            BlockKind::Material,
            Facing::North,
        );
        // 供电：检测器盯着平台，导线接到吸盘上方
        place(
            &mut world,
            IVec3::new(0, 1, 1),
            BlockKind::Detector,
            Facing::North,
        );
        place(&mut world, IVec3::new(1, 1, 1), BlockKind::Wire, Facing::North);

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_setup(world);

        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pushers,
            None,
            None,
        );

        assert!(
            world.is_material_at(IVec3::new(2, 1, 0)),
            "通电吸盘应拖住材料不下落"
        );
        assert!(!world.is_material_at(IVec3::new(2, 0, 0)));
    }

    /// 可动工厂上的吸盘吸附材料，阻拦器推出工厂 → 材料同移
    #[test]
    fn movable_suction_pushed_moves_adsorbed_material() {
        let mut world = WorldBlocks::default();
        // 阻拦器立柱（场景锚定）；不通电时阻拦器伸出推前方
        place(&mut world, IVec3::new(0, 0, 0), BlockKind::Stone, Facing::North);
        place(
            &mut world,
            IVec3::new(0, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(0, 2, 0),
            BlockKind::Blocker,
            Facing::East,
        );
        // 可动平台 + 吸盘 + 材料
        place(
            &mut world,
            IVec3::new(1, 2, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(2, 2, 0),
            BlockKind::SuctionCup,
            Facing::East,
        );
        place(
            &mut world,
            IVec3::new(3, 2, 0),
            BlockKind::Material,
            Facing::North,
        );
        // 可动结构自供电：向下传感器盯平台，导线接吸盘
        place(
            &mut world,
            IVec3::new(1, 3, 0),
            BlockKind::DownDetector,
            Facing::North,
        );
        place(&mut world, IVec3::new(2, 3, 0), BlockKind::Wire, Facing::North);

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_setup(world);

        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pushers,
            None,
            None,
        );

        assert!(
            world
                .blocks
                .get(&IVec3::new(2, 2, 0))
                .is_some_and(|b| b.kind == BlockKind::Platform),
            "平台应被推到 x=2"
        );
        assert!(
            world
                .blocks
                .get(&IVec3::new(3, 2, 0))
                .is_some_and(|b| b.kind == BlockKind::SuctionCup),
            "吸盘应随平台到 x=3"
        );
        assert!(
            world.is_material_at(IVec3::new(4, 2, 0)),
            "吸附材料应一起到 x=4"
        );
    }

    /// 传送带带动被吸材料 → 可动工厂一起走
    #[test]
    fn conveyor_moves_material_and_suction_factory_together() {
        let mut world = WorldBlocks::default();
        // 传送带（场景锚定）
        place(&mut world, IVec3::new(0, 0, 0), BlockKind::Stone, Facing::North);
        place(
            &mut world,
            IVec3::new(0, 1, 0),
            BlockKind::Conveyor,
            Facing::East,
        );
        // 材料在传送带上；吸盘在材料南侧朝北吸附；平台贴在吸盘南侧
        place(
            &mut world,
            IVec3::new(0, 2, 0),
            BlockKind::Material,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(0, 2, 1),
            BlockKind::SuctionCup,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(0, 2, 2),
            BlockKind::Platform,
            Facing::North,
        );
        // 供电：固定平台作检测目标；导线 Detector(2,1,1)-Wire(1,2,1)-Suction(0,2,1)
        place(&mut world, IVec3::new(2, 0, 0), BlockKind::Stone, Facing::North);
        place(
            &mut world,
            IVec3::new(2, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(2, 1, 1),
            BlockKind::Detector,
            Facing::North,
        );
        place(&mut world, IVec3::new(2, 2, 1), BlockKind::Wire, Facing::North);
        place(&mut world, IVec3::new(1, 2, 1), BlockKind::Wire, Facing::North);

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_setup(world);

        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pushers,
            None,
            None,
        );

        assert!(
            world.is_material_at(IVec3::new(1, 2, 0)),
            "材料应被传送带东移"
        );
        assert!(
            world
                .blocks
                .get(&IVec3::new(1, 2, 1))
                .is_some_and(|b| b.kind == BlockKind::SuctionCup),
            "吸盘应随材料一起东移"
        );
        assert!(
            world
                .blocks
                .get(&IVec3::new(1, 2, 2))
                .is_some_and(|b| b.kind == BlockKind::Platform),
            "平台应随吸盘一起东移"
        );
    }

    #[test]
    fn unpowered_suction_does_not_hold_material() {
        let mut world = WorldBlocks::default();
        place(&mut world, IVec3::new(0, 0, 0), BlockKind::Stone, Facing::North);
        place(
            &mut world,
            IVec3::new(0, 1, 0),
            BlockKind::Platform,
            Facing::North,
        );
        place(
            &mut world,
            IVec3::new(1, 1, 0),
            BlockKind::SuctionCup,
            Facing::East,
        );
        place(
            &mut world,
            IVec3::new(2, 1, 0),
            BlockKind::Material,
            Facing::North,
        );

        let (mut world, mut pending, mut signals, mut structures, mut influence, mut pushers) =
            sim_setup(world);

        simulate_turn(
            &mut world,
            &mut pending,
            &mut signals,
            1,
            &mut structures,
            &mut influence,
            &mut pushers,
            None,
            None,
        );

        assert!(
            !world.is_material_at(IVec3::new(2, 1, 0)),
            "未通电时材料应下落"
        );
        assert!(world.is_material_at(IVec3::new(2, 0, 0)));
    }
}
