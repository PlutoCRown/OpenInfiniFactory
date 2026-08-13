use crate::simulation::core::{TurnOutput, simulate_turn};
use crate::simulation::movement::PusherState;
use crate::simulation::pending::PendingGeneratedMaterials;
use crate::simulation::signals::SignalNetworkCache;
use crate::simulation::stats::SimulationStepStats;
use crate::simulation::structure_state::StructureState;
use crate::simulation::structures::MovementInfluenceCache;
use crate::world::grid::WorldBlocks;

use super::SimulationDebugLog;
use super::control::SimulationControl;

/// 自有模拟会话：世界与回合状态，无 Bevy App
pub struct SimSession {
    pub world: WorldBlocks,
    pub pending_generated: PendingGeneratedMaterials,
    pub signal_cache: SignalNetworkCache,
    pub structure_state: StructureState,
    pub movement_influence: MovementInfluenceCache,
    pub pusher_state: PusherState,
    pub control: SimulationControl,
    pub log: SimulationDebugLog,
    pub stats: SimulationStepStats,
}

impl SimSession {
    /// 新建空会话
    pub fn new() -> Self {
        Self {
            world: WorldBlocks::default(),
            pending_generated: PendingGeneratedMaterials::default(),
            signal_cache: SignalNetworkCache::default(),
            structure_state: StructureState::default(),
            movement_influence: MovementInfluenceCache::default(),
            pusher_state: PusherState::default(),
            control: SimulationControl::default(),
            log: SimulationDebugLog::default(),
            stats: SimulationStepStats::default(),
        }
    }

    /// 只读世界网格
    pub fn world_blocks(&self) -> &WorldBlocks {
        &self.world
    }

    /// 可变世界网格
    pub fn world_blocks_mut(&mut self) -> &mut WorldBlocks {
        &mut self.world
    }

    /// 是否处于模拟中（运行或已推进过回合）
    pub fn is_active(&self) -> bool {
        self.control.is_active()
    }

    /// 只读控制面
    pub fn control(&self) -> &SimulationControl {
        &self.control
    }

    /// 开局：快照世界与结构，重建推杆状态
    pub fn begin_simulation(&mut self) {
        self.control.begin(
            &self.world,
            &mut self.structure_state,
            &mut self.pusher_state,
        );
    }

    /// 请求连续跑回合
    pub fn request_continuous_run(&mut self) {
        self.control.run(
            &self.world,
            &mut self.structure_state,
            &mut self.pusher_state,
        );
    }

    /// 请求单步（须已激活）
    pub fn request_one_turn(&mut self) -> Result<(), &'static str> {
        self.control.step()
    }

    /// 回滚到开局检查点
    pub fn rollback(&mut self) {
        self.control.rollback(
            &mut self.world,
            &mut self.pending_generated,
            &mut self.structure_state,
            &mut self.movement_influence,
            &mut self.pusher_state,
        );
    }

    /// 清空会话到默认空世界
    pub fn reset(&mut self) {
        self.rollback();
        self.world = WorldBlocks::default();
        self.pending_generated.clear();
        self.signal_cache = SignalNetworkCache::default();
        self.structure_state.clear();
        self.movement_influence.clear();
        self.pusher_state.clear();
        self.control.reset();
    }

    /// 推进下一回合并更新控制面回合计数
    pub fn simulate_next_turn(&mut self) -> TurnOutput {
        self.simulate_next_turn_with_logging(true)
    }

    /// 推进下一回合；批量性能测试可关闭逐块日志，避免观测本身污染耗时
    pub fn simulate_next_turn_with_logging(&mut self, logging: bool) -> TurnOutput {
        let next_turn = self.control.turn + 1;
        let output = simulate_turn(
            &mut self.world,
            &mut self.pending_generated,
            &mut self.signal_cache,
            next_turn,
            &mut self.structure_state,
            &mut self.movement_influence,
            &mut self.pusher_state,
            logging.then_some(&mut self.log),
            Some(&mut self.stats),
        );
        self.control.turn = next_turn;
        output
    }
}

impl Default for SimSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind};
    use crate::world::Facing;
    use glam::IVec3;

    /// 粘头伸出：身前平台被推走后，头格出现 PusherHead，且整坨仍可通过臂连通
    #[test]
    fn sticky_blocker_extend_places_head_without_orphan_gap() {
        let mut session = SimSession::new();
        let back = IVec3::new(0, 1, 0);
        let body = IVec3::new(1, 1, 0);
        let face = IVec3::new(2, 1, 0);
        session
            .world
            .insert(back, BlockData::new(BlockKind::Platform, Facing::North));
        session
            .world
            .insert(body, BlockData::new(BlockKind::Blocker, Facing::East));
        session
            .world
            .insert(face, BlockData::new(BlockKind::Platform, Facing::North));

        session.begin_simulation();
        session.simulate_next_turn();

        let head_block = session.world.blocks.get(&face);
        assert!(
            head_block.is_some_and(|b| b.kind == BlockKind::PusherHead),
            "face should become PusherHead, got {:?}",
            head_block.map(|b| b.kind)
        );
        let front = face + IVec3::X;
        assert!(
            session.world.is_factory_at(front),
            "platform should be pushed to {front}"
        );
        // 体仍在原位
        assert_eq!(
            session.world.blocks.get(&body).map(|b| b.kind),
            Some(BlockKind::Blocker)
        );
        // 重建后仍同结构（穿过活塞臂）
        session
            .structure_state
            .rebuild_for_simulation(&session.world);
        let sid_body = session.structure_state.structure_id_at(body);
        let sid_front = session.structure_state.structure_id_at(front);
        assert_eq!(
            sid_body, sid_front,
            "body and front must remain one structure through the arm"
        );
    }

    /// 整排直接锚定的拦截器只允许空头伸出，不为不可动的内部成员构建变形
    #[test]
    fn grounded_blocker_row_only_extends_into_empty_head() {
        let mut session = SimSession::new();
        for x in 0..3 {
            session.world.insert(
                IVec3::new(x, 0, 0),
                BlockData::new(BlockKind::Scene(crate::blocks::SceneBlockId(0)), Facing::North),
            );
            session.world.insert(
                IVec3::new(x, 1, 0),
                BlockData::new(BlockKind::Blocker, Facing::East),
            );
        }

        session.begin_simulation();
        session.simulate_next_turn();

        assert_eq!(
            session
                .structure_state
                .get(session.structure_state.id_at(IVec3::new(0, 1, 0)).unwrap())
                .map(|structure| structure.deform_groups.len()),
            Some(0)
        );
        assert_eq!(
            session.world.blocks.get(&IVec3::new(3, 1, 0)).map(|block| block.kind),
            Some(BlockKind::PusherHead)
        );
        for x in 0..3 {
            assert_eq!(
                session.world.blocks.get(&IVec3::new(x, 1, 0)).map(|block| block.kind),
                Some(BlockKind::Blocker)
            );
        }
    }
}
