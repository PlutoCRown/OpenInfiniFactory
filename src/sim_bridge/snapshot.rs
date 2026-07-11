use crate::game::simulation::movement::PusherState;
use crate::game::simulation::pending::PendingGeneratedMaterials;
use crate::game::simulation::signals::SignalNetworkCache;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::world::grid::WorldBlocks;

use super::TurnOutput;

/// 预取用的模拟世界快照（存 oif-sim 内层类型）
#[derive(Clone)]
pub struct SimSnapshot {
    pub world: oif_sim::WorldBlocks,
    pub pending_generated: oif_sim::simulation::pending::PendingGeneratedMaterials,
    pub signal_cache: oif_sim::simulation::signals::SignalNetworkCache,
    pub structure_state: oif_sim::simulation::structure_state::StructureState,
    pub movement_influence: oif_sim::simulation::structures::MovementInfluenceCache,
    pub pusher_state: oif_sim::simulation::movement::PusherState,
}

/// 预计算完成的一回合
#[derive(Clone)]
pub struct CachedTurn {
    pub output: TurnOutput,
    pub after: SimSnapshot,
}

impl SimSnapshot {
    /// 从游戏侧 Resource 包装拷贝内层状态
    pub fn from_world(
        world: &WorldBlocks,
        pending_generated: &PendingGeneratedMaterials,
        signal_cache: &SignalNetworkCache,
        structure_state: &StructureState,
        movement_influence: &MovementInfluenceCache,
        pusher_state: &PusherState,
    ) -> Self {
        Self {
            world: world.0.clone(),
            pending_generated: pending_generated.0.clone(),
            signal_cache: signal_cache.0.clone(),
            structure_state: structure_state.0.clone(),
            movement_influence: movement_influence.0.clone(),
            pusher_state: pusher_state.0.clone(),
        }
    }
}
