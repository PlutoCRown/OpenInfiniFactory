use glam::IVec3;
use std::collections::HashMap;

use crate::blocks::BlockData;

/// 跨回合挂起的生成材料（传送/销毁不再跨回合 pending）
#[derive(Default, Clone)]
pub struct PendingGeneratedMaterials {
    pending: HashMap<IVec3, PendingGeneratedMaterial>,
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub(crate) fn pending_keys(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.pending.keys().copied()
    }

    pub(crate) fn insert_pending(&mut self, pos: IVec3, block: BlockData, ready_turn: u64) {
        self.pending
            .entry(pos)
            .or_insert(PendingGeneratedMaterial { block, ready_turn });
    }

    pub(crate) fn ready_pending_positions(&self, turn: u64) -> Vec<IVec3> {
        self.pending
            .iter()
            .filter_map(|(pos, pending)| (pending.ready_turn <= turn).then_some(*pos))
            .collect()
    }

    pub(crate) fn take_pending_block(&mut self, pos: IVec3) -> Option<BlockData> {
        self.pending.remove(&pos).map(|pending| pending.block)
    }

    pub fn pending_entries(&self) -> impl Iterator<Item = (IVec3, BlockData, u64)> + '_ {
        self.pending
            .iter()
            .map(|(pos, pending)| (*pos, pending.block, pending.ready_turn))
    }
}

#[derive(Clone)]
struct PendingGeneratedMaterial {
    block: BlockData,
    ready_turn: u64,
}
