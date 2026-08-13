use crate::simulation::movement::PusherState;
use crate::simulation::pending::PendingGeneratedMaterials;
use crate::simulation::structure_state::StructureState;
use crate::simulation::structures::MovementInfluenceCache;
use crate::world::grid::WorldBlocks;

/// 游戏与无头调试共用的模拟控制面：回合计数、运行意图与回滚检查点
#[derive(bevy_ecs::prelude::Resource, Clone)]
pub struct SimulationControl {
    pub turn: u64,
    pub running: bool,
    pub step_requested: bool,
    pub speed: f32,
    pub accumulator: f32,
    pub start_snapshot: Option<WorldBlocks>,
    pub start_structures: Option<StructureState>,
}

impl SimulationControl {
    /// 是否处于模拟中
    pub fn is_active(&self) -> bool {
        self.start_snapshot.is_some() || self.running || self.turn > 0
    }

    /// 返回编辑态世界；模拟中读取开局检查点
    pub fn authoring_world<'a>(&'a self, current: &'a WorldBlocks) -> &'a WorldBlocks {
        self.start_snapshot.as_ref().unwrap_or(current)
    }

    /// 清空控制状态，进入未开始的编辑态
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 开始模拟并记录唯一的回滚检查点
    pub fn begin(
        &mut self,
        world: &WorldBlocks,
        structure_state: &mut StructureState,
        pusher_state: &mut PusherState,
    ) {
        if self.is_active() {
            return;
        }
        self.start_snapshot = Some(world.clone());
        *pusher_state = PusherState::rebuild_from_world(world);
        structure_state.refresh_for_simulation_start(world);
        self.start_structures = Some(structure_state.clone());
    }

    /// 开始或恢复连续推进
    pub fn run(
        &mut self,
        world: &WorldBlocks,
        structure_state: &mut StructureState,
        pusher_state: &mut PusherState,
    ) {
        self.begin(world, structure_state, pusher_state);
        self.running = true;
    }

    /// 暂停推进并取消排队的单步
    pub fn pause(&mut self) {
        self.running = false;
        self.step_requested = false;
        self.speed = 1.0;
    }

    /// 请求推进一回合
    pub fn step(&mut self) -> Result<(), &'static str> {
        if !self.is_active() {
            return Err("simulation is not active");
        }
        self.running = false;
        self.speed = 1.0;
        self.step_requested = true;
        Ok(())
    }

    /// 回滚权威模拟状态到开局检查点
    pub fn rollback(
        &mut self,
        world: &mut WorldBlocks,
        pending_generated: &mut PendingGeneratedMaterials,
        structure_state: &mut StructureState,
        movement_influence: &mut MovementInfluenceCache,
        pusher_state: &mut PusherState,
    ) {
        self.running = false;
        self.step_requested = false;
        self.turn = 0;
        self.accumulator = 0.0;
        pending_generated.clear();
        movement_influence.clear();
        pusher_state.clear();
        let factory_snapshot = self.start_structures.take();
        if let Some(snapshot) = self.start_snapshot.take() {
            *world = snapshot;
        } else {
            world.retain(|_, block| !block.kind.is_material());
            world.clear_generated_markers();
        }
        if let Some(snapshot) = factory_snapshot {
            *structure_state = snapshot;
        } else {
            structure_state.rebuild_for_runtime(world);
        }
    }
}

impl Default for SimulationControl {
    fn default() -> Self {
        Self {
            turn: 0,
            running: false,
            step_requested: false,
            speed: 1.0,
            accumulator: 0.0,
            start_snapshot: None,
            start_structures: None,
        }
    }
}
