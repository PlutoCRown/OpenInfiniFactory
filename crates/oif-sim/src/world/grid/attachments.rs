//! 附着、焊接与电线面灯面板

use glam::IVec3;

use crate::blocks::{BlockData, BlockId, BlockKind};

use super::{MaterialAttachment, MaterialFace, MaterialWeld, WorldBlocks};

impl WorldBlocks {
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
        // 仅搬迁 blocks 层自有 settings（如告示）；同格系统块（传送门）的 settings 不动
        let mut moved_settings = Vec::new();
        for (from, to, _) in &moves {
            if self.system_blocks.contains_key(from) {
                continue;
            }
            if let Some(settings) = self.block_settings.remove(from) {
                moved_settings.push((*to, settings));
            }
        }
        for (from, _, _) in &moves {
            self.blocks.remove(from);
        }
        for (_, to, block) in moves {
            self.blocks.insert(to, block);
        }
        for (to, settings) in moved_settings {
            if self.system_blocks.contains_key(&to) {
                continue;
            }
            self.block_settings.insert(to, settings);
        }
        if touches_signals {
            self.topology_revision = self.topology_revision.wrapping_add(1);
        }
    }

    /// 宿主面是否允许贴告示：场景任意面；工厂非 non_connection；材料 Connectable
    pub fn host_face_accepts_sign(host: &BlockData, face_normal: IVec3) -> bool {
        if host.kind.is_scene() {
            return true;
        }
        if host.kind.is_factory() {
            return host.kind.face_attachable(host.facing, face_normal);
        }
        if host.kind.is_material() {
            return host
                .kind
                .material_face_connectable(host.facing, face_normal);
        }
        false
    }

    /// 在 host 面邻格放置告示是否合法（格空 + 面门禁）
    pub fn can_place_sign_on_face(&self, host_pos: IVec3, face_normal: IVec3) -> bool {
        if face_normal == IVec3::ZERO || face_normal.abs().element_sum() != 1 {
            return false;
        }
        let Some(host) = self.blocks.get(&host_pos) else {
            return false;
        };
        if !Self::host_face_accepts_sign(host, face_normal) {
            return false;
        }
        let place_at = host_pos + face_normal;
        place_at.y >= 0 && self.can_place_block_kind_at(place_at, BlockKind::Sign)
    }

    /// 写入告示工厂附着（父须有 BlockId；场景宿主不记附着）
    pub fn attach_factory_child(
        &mut self,
        child_id: BlockId,
        parent_id: BlockId,
        parent_face_normal: IVec3,
    ) {
        if child_id.is_none() || parent_id.is_none() {
            return;
        }
        self.factory_attachments.insert(
            child_id,
            MaterialAttachment {
                parent: parent_id,
                parent_face_normal,
            },
        );
    }

    /// 按几何重建告示附着（加载后调用）
    pub fn rebuild_factory_attachments(&mut self) {
        self.factory_attachments.clear();
        let signs: Vec<(IVec3, BlockData)> = self
            .blocks
            .iter()
            .filter(|(_, block)| block.kind == BlockKind::Sign && !block.id.is_none())
            .map(|(pos, block)| (*pos, *block))
            .collect();
        for (pos, sign) in signs {
            let candidates = [
                pos - sign.facing.forward_ivec3(),
                pos + IVec3::NEG_Y,
                pos + IVec3::X,
                pos + IVec3::NEG_X,
                pos + IVec3::Z,
                pos + IVec3::NEG_Z,
                pos + IVec3::Y,
            ];
            for host_pos in candidates {
                let Some(host) = self.blocks.get(&host_pos).copied() else {
                    continue;
                };
                if host.id.is_none() {
                    continue;
                }
                let normal = pos - host_pos;
                if normal.abs().element_sum() != 1 {
                    continue;
                }
                if !Self::host_face_accepts_sign(&host, normal) {
                    continue;
                }
                self.attach_factory_child(sign.id, host.id, normal);
                break;
            }
        }
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
        if self
            .material_welds
            .insert(MaterialWeld::new(block_a.id, block_b.id))
        {
            self.topology_revision = self.topology_revision.wrapping_add(1);
            true
        } else {
            false
        }
    }
}
