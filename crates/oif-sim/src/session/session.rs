use crate::simulation::core::{simulate_turn, TurnOutput};
use crate::simulation::movement::PusherState;
use crate::simulation::pending::PendingGeneratedMaterials;
use crate::simulation::signals::SignalNetworkCache;
use crate::simulation::stats::SimulationStepStats;
use crate::simulation::structure_state::StructureState;
use crate::simulation::structures::MovementInfluenceCache;
use crate::world::grid::WorldBlocks;

use super::control::SimulationControl;
use super::SimulationDebugLog;

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
        if self.is_active() {
            return;
        }
        self.control.start_snapshot = Some(self.world.clone());
        self.pusher_state = PusherState::rebuild_from_world(&self.world);
        self.structure_state.rebuild_for_simulation(&self.world);
        self.control.start_structures = Some(self.structure_state.clone());
    }

    /// 请求连续跑回合
    pub fn request_continuous_run(&mut self) {
        self.begin_simulation();
        self.control.running = true;
    }

    /// 请求单步（须已激活）
    pub fn request_one_turn(&mut self) -> Result<(), &'static str> {
        if !self.is_active() {
            return Err("simulation is not active");
        }
        self.control.running = false;
        self.control.speed = 1.0;
        self.control.step_requested = true;
        Ok(())
    }

    /// 回滚到开局快照；返回开局结构状态
    pub fn rollback(&mut self) -> Option<StructureState> {
        self.control.running = false;
        self.control.step_requested = false;
        self.control.turn = 0;
        self.control.accumulator = 0.0;
        self.pending_generated.clear();
        self.movement_influence.clear();
        self.pusher_state.clear();
        let factory_snapshot = self.control.start_structures.take();
        if let Some(snapshot) = self.control.start_snapshot.take() {
            self.world = snapshot;
        } else {
            self.world.retain(|_, block| !block.kind.is_material());
            self.world.clear_generated_markers();
        }
        factory_snapshot
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
        self.control.turn = 0;
        self.control.running = false;
        self.control.step_requested = false;
        self.control.accumulator = 0.0;
        self.control.speed = 1.0;
    }

    /// 推进下一回合并更新控制面回合计数
    pub fn simulate_next_turn(&mut self) -> TurnOutput {
        let next_turn = self.control.turn + 1;
        let output = simulate_turn(
            &mut self.world,
            &mut self.pending_generated,
            &mut self.signal_cache,
            next_turn,
            &mut self.structure_state,
            &mut self.movement_influence,
            &mut self.pusher_state,
            Some(&mut self.log),
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
