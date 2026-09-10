use glam::IVec3;
use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockData, BlockId, BlockKind, PaintMaterialId, StampMaterialId};
use crate::world::grid::MaterialFace;

/// 跨回合挂起：生成、延后销毁、延后漆/印花（等移动动画播完再落地）
#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct PendingTurnEffects {
    pending: HashMap<IVec3, PendingGeneratedMaterial>,
    /// 钻头/验收销毁：本回合只标记，下一回合开始再移除
    pending_destroyed: HashMap<IVec3, PendingDestroyedMaterial>,
    /// 滚刷漆：本回合只标记，下一回合开始再写入
    pending_paints: HashMap<MaterialFace, PendingPaint>,
    /// 印花：按印花机格挂起，下一回合开始再生成附着
    pending_stamps: HashMap<IVec3, PendingStamp>,
}

/// 延后销毁原因（立即销毁如激光/脆弱不走挂起表）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingDestroyReason {
    Drill,
    Accept,
}

impl PendingTurnEffects {
    pub fn clear(&mut self) {
        self.pending.clear();
        self.pending_destroyed.clear();
        self.pending_paints.clear();
        self.pending_stamps.clear();
    }

    /// 回合提交下一次生成操作；编辑预览不得写入这份队列
    pub fn schedule_generation(
        &mut self,
        world: &crate::world::grid::WorldBlocks,
        ready_turn: u64,
        accepted_acceptors: &HashSet<crate::blocks::AcceptorId>,
    ) {
        self.pending = super::planned_generation(world, ready_turn, accepted_acceptors)
            .into_iter()
            .map(|generated| {
                (
                    generated.pos,
                    PendingGeneratedMaterial {
                        block: generated.block,
                        ready_turn,
                    },
                )
            })
            .collect();
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

    pub(crate) fn mark_paint(
        &mut self,
        face: MaterialFace,
        paint: PaintMaterialId,
        ready_turn: u64,
    ) {
        self.pending_paints
            .entry(face)
            .or_insert(PendingPaint { paint, ready_turn });
    }

    pub(crate) fn mark_stamp(
        &mut self,
        stamper_pos: IVec3,
        host: BlockId,
        face_normal: IVec3,
        stamp: StampMaterialId,
        ready_turn: u64,
    ) {
        self.pending_stamps
            .entry(stamper_pos)
            .or_insert(PendingStamp {
                host,
                face_normal,
                stamp,
                ready_turn,
            });
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

    /// 取出本回合应写入的延后漆
    pub(crate) fn take_ready_paints(&mut self, turn: u64) -> Vec<(MaterialFace, PaintMaterialId)> {
        let ready: Vec<MaterialFace> = self
            .pending_paints
            .iter()
            .filter_map(|(face, pending)| (pending.ready_turn <= turn).then_some(*face))
            .collect();
        let mut out = Vec::with_capacity(ready.len());
        for face in ready {
            if let Some(pending) = self.pending_paints.remove(&face) {
                out.push((face, pending.paint));
            }
        }
        out
    }

    /// 取出本回合应生成的延后印花
    pub(crate) fn take_ready_stamps(&mut self, turn: u64) -> Vec<(IVec3, PendingStamp)> {
        let ready: Vec<IVec3> = self
            .pending_stamps
            .iter()
            .filter_map(|(pos, pending)| (pending.ready_turn <= turn).then_some(*pos))
            .collect();
        let mut out = Vec::with_capacity(ready.len());
        for pos in ready {
            if let Some(pending) = self.pending_stamps.remove(&pos) {
                out.push((pos, pending));
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
struct PendingPaint {
    paint: PaintMaterialId,
    ready_turn: u64,
}

/// 延后印花：挂在宿主材料面，落地时写入面附着
#[derive(Clone)]
pub(crate) struct PendingStamp {
    pub host: BlockId,
    pub face_normal: IVec3,
    pub stamp: StampMaterialId,
    ready_turn: u64,
}

/// 编辑预览和跨回合操作的生命周期回归
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::MaterialBlockId;
    use crate::world::Facing;
    use crate::world::grid::{GeneratorMode, GeneratorSettings, WorldBlocks};

    /// 编辑预览读取最新配置但不改已提交队列，重新提交也不保留删除的源
    #[test]
    fn preview_is_independent_from_committed_generation() {
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::ZERO,
            BlockData::new(BlockKind::Generator, Facing::North),
        );
        let mut settings = GeneratorSettings {
            mode: GeneratorMode::Period {
                period: 1,
                offset: 0,
            },
            material: MaterialBlockId(0),
            facing: Facing::North,
        };
        world.set_generator_settings(IVec3::ZERO, settings);
        let mut pending = PendingTurnEffects::default();
        pending.schedule_generation(&world, 2, &HashSet::new());
        settings.facing = Facing::East;
        world.set_generator_settings(IVec3::ZERO, settings);
        let preview = super::super::planned_generation(&world, 1, &HashSet::new());
        assert_eq!(preview[0].block.facing, Facing::East);
        assert_eq!(
            pending.pending_entries().next().unwrap().1.facing,
            Facing::North
        );
        world.remove_system(&IVec3::ZERO);
        assert!(super::super::planned_generation(&world, 1, &HashSet::new()).is_empty());
        pending.schedule_generation(&world, 3, &HashSet::new());
        assert_eq!(pending.pending_entries().count(), 0);
    }
}
