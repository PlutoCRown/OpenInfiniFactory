use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::systems::debug::DebugState;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{BlockEntity, WorldRenderAssets};

/// ECS access bundle for mutating the loaded playing world and its render/sim sidecars.
#[derive(SystemParam)]
pub struct PlayingWorldParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub debug: Res<'w, DebugState>,
    pub structure_state: ResMut<'w, StructureState>,
    pub movement_influence: ResMut<'w, MovementInfluenceCache>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
}
