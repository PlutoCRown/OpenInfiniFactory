//! 占用、放置与运动进入判定

use glam::IVec3;

use crate::blocks::{BlockId, BlockKind};

use super::WorldBlocks;

impl WorldBlocks {
    /// 模拟占用：与玩家碰撞 `has_collision` 分离；blocks 层有块即占格
    pub fn is_occupied(&self, pos: IVec3) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| block.kind.has_collision())
            || self.blocks.contains_key(&pos)
    }

    /// 平台/工厂放置占用（blocks 层有块即占）
    pub fn is_platform_occupied(&self, pos: IVec3) -> bool {
        self.blocks.contains_key(&pos)
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

    /// 从 from 格移入 target 时是否可进入（含脆弱材料让出）
    pub fn cell_accepts_move_from(&self, _from: IVec3, target: IVec3) -> bool {
        self.can_move_into(target) || self.is_fragile_material_at(target)
    }

    /// 格上是否为脆弱材料（运动冲突时让出并碎裂）
    pub fn is_fragile_material_at(&self, pos: IVec3) -> bool {
        self.blocks.get(&pos).is_some_and(|block| {
            block
                .kind
                .material_props()
                .is_some_and(|props| props.fragile)
        })
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

    pub fn is_teleport_at(&self, pos: IVec3) -> bool {
        self.system_blocks.get(&pos).is_some_and(|block| {
            block
                .kind
                .material_processor()
                .is_some_and(|processor| processor.is_teleport())
        })
    }

    pub fn anchors_material_at_teleport(&self, pos: IVec3) -> bool {
        self.is_material_at(pos) && self.is_teleport_at(pos)
    }

    /// 标记材料刚传送到达（同 id 留在口内时下回合不传；换 id / 变空则清标记）
    pub fn mark_teleport_arrival(&mut self, pos: IVec3, id: BlockId) {
        self.teleport_arrivals.insert(pos, id);
    }

    /// 材料是否仍是本口记录的「刚到达」方块
    pub fn is_teleport_arrival(&self, pos: IVec3, id: BlockId) -> bool {
        self.teleport_arrivals.get(&pos) == Some(&id)
    }

    /// 同步到达标记：空则清除；仍是原 id 则保留；已换成别的 id 则清除以便尝试传送
    pub fn sync_teleport_arrivals(&mut self) {
        self.teleport_arrivals.retain(|pos, id| {
            self.blocks
                .get(pos)
                .is_some_and(|block| block.kind.is_material() && block.id == *id)
        });
    }

    /// 标记旋转器刚转过工作面上的材料（同 id 留着则不连转）
    pub fn mark_rotator_arrival(&mut self, rotator_pos: IVec3, id: BlockId) {
        self.rotator_arrivals.insert(rotator_pos, id);
    }

    /// 工作面材料是否仍是该旋转器记录的「刚转过」方块
    pub fn is_rotator_arrival(&self, rotator_pos: IVec3, id: BlockId) -> bool {
        self.rotator_arrivals.get(&rotator_pos) == Some(&id)
    }

    /// 同步旋转锁：工作面（rotator+Y）空/换 id 则清
    pub fn sync_rotator_arrivals(&mut self) {
        self.rotator_arrivals.retain(|rotator_pos, id| {
            self.blocks
                .get(&(*rotator_pos + IVec3::Y))
                .is_some_and(|block| {
                    (block.kind.is_material() || block.kind.is_factory()) && block.id == *id
                })
        });
    }

    pub fn is_factory_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_factory())
    }

    pub fn is_detectable_by_detector_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_detector_target())
    }

    pub fn is_scene_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_scene())
    }
}
