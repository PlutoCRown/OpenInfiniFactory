//! 占用、放置与运动进入判定

use glam::IVec3;

use crate::blocks::{BlockData, BlockKind};

use super::WorldBlocks;

impl WorldBlocks {
    pub fn is_occupied(&self, pos: IVec3) -> bool {
        self.system_blocks
            .get(&pos)
            .is_some_and(|block| block.kind.has_collision())
            || self
                .blocks
                .get(&pos)
                .is_some_and(|block| block.kind.has_collision())
            || self
                .machine_bodies
                .get(&pos)
                .is_some_and(|block| block.kind.has_collision())
    }

    pub fn is_platform_occupied(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.has_collision())
            || self
                .machine_bodies
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
            || self.machine_bodies.contains_key(&pos)
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

    /// 机身是否允许该印花材料沿工作朝向进入本格
    pub fn stamper_body_allows_stamp(&self, pos: IVec3, stamp: &BlockData) -> bool {
        let Some(body) = self.machine_bodies.get(&pos) else {
            return false;
        };
        if !body.kind.allows_stamp_passthrough() {
            return false;
        }
        if !stamp
            .kind
            .material_props()
            .is_some_and(|props| props.is_stamp)
        {
            return false;
        }
        let Some(att) = self.material_attachments.get(&stamp.id) else {
            return false;
        };
        // 附着法线从宿主指向印花；机身 facing 指向宿主，故两者反向
        att.parent_face_normal == -body.facing.forward_ivec3()
    }

    /// 从 from 格移入 target 时是否可进入（含印花对 StamperBody 透传、脆弱让出）
    pub fn cell_accepts_move_from(&self, from: IVec3, target: IVec3) -> bool {
        if self.can_move_into(target) || self.is_fragile_material_at(target) {
            return true;
        }
        self.blocks
            .get(&from)
            .is_some_and(|mover| self.stamper_body_allows_stamp(target, mover))
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
            .is_some_and(|block| block.kind.is_detector_target())
    }

    pub fn is_scene_at(&self, pos: IVec3) -> bool {
        self.blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_scene())
    }
}
