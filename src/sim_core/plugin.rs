use bevy::prelude::*;

use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::{PendingGeneratedMaterials, SignalNetworkCache};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::world::grid::WorldBlocks;

use super::control::SimulationControl;
use super::SimulationDebugLog;

impl Resource for SimulationControl {}
impl Resource for SimulationDebugLog {}

pub struct SimCorePlugin;

impl Plugin for SimCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldBlocks>()
            .init_resource::<PendingGeneratedMaterials>()
            .init_resource::<SignalNetworkCache>()
            .init_resource::<StructureState>()
            .init_resource::<MovementInfluenceCache>()
            .init_resource::<PusherState>()
            .init_resource::<SimulationControl>()
            .init_resource::<SimulationDebugLog>();
    }
}
