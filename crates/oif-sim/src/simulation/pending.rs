use glam::IVec3;
use std::collections::HashMap;

use crate::blocks::{BlockData, BlockKind};

/// 跨回合挂起：生成、延后销毁、延后传送（等移动动画播完再落地）
#[derive(Default, Clone)]
pub struct PendingGeneratedMaterials {
    pending: HashMap<IVec3, PendingGeneratedMaterial>,
    /// 钻头/验收销毁：本回合只标记，下一回合开始再移除
    pending_destroyed: HashMap<IVec3, PendingDestroyedMaterial>,
    /// 传送：本回合只标记入口→出口，下一回合开始再搬迁
    pending_teleports: HashMap<IVec3, PendingTeleport>,
}

/// 延后销毁原因
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingDestroyReason {
    Drill,
    Accept,
}

impl PendingGeneratedMaterials {
    pub fn clear(&mut self) {
        self.pending.clear();
        self.pending_destroyed.clear();
        self.pending_teleports.clear();
    }

    pub(crate) fn pending_keys(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.pending.keys().copied()
    }

    pub(crate) fn insert_pending(&mut self, pos: IVec3, block: BlockData, ready_turn: u64) {
        self.pending
            .entry(pos)
            .or_insert(PendingGeneratedMaterial { block, ready_turn });
    }

    pub(crate) fn mark_destroyed(
        &mut self,
        pos: IVec3,
        kind: BlockKind,
        ready_turn: u64,
        reason: PendingDestroyReason,
    ) {
        self.pending_destroyed
            .entry(pos)
            .or_insert(PendingDestroyedMaterial {
                kind,
                ready_turn,
                reason,
            });
    }

    pub(crate) fn mark_teleport(&mut self, entrance: IVec3, exit: IVec3, ready_turn: u64) {
        self.pending_teleports
            .entry(entrance)
            .or_insert(PendingTeleport { exit, ready_turn });
    }

    /// 取出本回合应落地的延后销毁（并从挂起表移除）
    pub(crate) fn take_ready_destroyed(
        &mut self,
        turn: u64,
    ) -> Vec<(IVec3, BlockKind, PendingDestroyReason)> {
        let ready: Vec<IVec3> = self
            .pending_destroyed
            .iter()
            .filter_map(|(pos, pending)| (pending.ready_turn <= turn).then_some(*pos))
            .collect();
        let mut out = Vec::with_capacity(ready.len());
        for pos in ready {
            if let Some(pending) = self.pending_destroyed.remove(&pos) {
                out.push((pos, pending.kind, pending.reason));
            }
        }
        out
    }

    /// 取出本回合应落地的延后传送（入口 → 出口）
    pub(crate) fn take_ready_teleports(&mut self, turn: u64) -> Vec<(IVec3, IVec3)> {
        let ready: Vec<IVec3> = self
            .pending_teleports
            .iter()
            .filter_map(|(entrance, pending)| (pending.ready_turn <= turn).then_some(*entrance))
            .collect();
        let mut out = Vec::with_capacity(ready.len());
        for entrance in ready {
            if let Some(pending) = self.pending_teleports.remove(&entrance) {
                out.push((entrance, pending.exit));
            }
        }
        out
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

#[derive(Clone)]
struct PendingDestroyedMaterial {
    kind: BlockKind,
    ready_turn: u64,
    reason: PendingDestroyReason,
}

#[derive(Clone)]
struct PendingTeleport {
    exit: IVec3,
    ready_turn: u64,
}
