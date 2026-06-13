use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::{PendingGeneratedMaterials, SignalNetworkCache};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::world::grid::WorldBlocks;

use super::TurnOutput;

#[derive(Clone)]
pub struct SimSnapshot {
    pub world: WorldBlocks,
    pub pending_generated: PendingGeneratedMaterials,
    pub signal_cache: SignalNetworkCache,
    pub structure_state: StructureState,
    pub movement_influence: MovementInfluenceCache,
    pub pusher_state: PusherState,
}

#[derive(Clone)]
pub struct CachedTurn {
    pub output: TurnOutput,
    pub after: SimSnapshot,
}

impl SimSnapshot {
    pub fn from_world(
        world: &WorldBlocks,
        pending_generated: &PendingGeneratedMaterials,
        signal_cache: &SignalNetworkCache,
        structure_state: &StructureState,
        movement_influence: &MovementInfluenceCache,
        pusher_state: &PusherState,
    ) -> Self {
        Self {
            world: world.clone(),
            pending_generated: pending_generated.clone(),
            signal_cache: signal_cache.clone(),
            structure_state: structure_state.clone(),
            movement_influence: movement_influence.clone(),
            pusher_state: pusher_state.clone(),
        }
    }
}
