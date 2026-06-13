use bevy::ecs::system::SystemState;
use bevy::prelude::*;

use crate::game::simulation::core::{simulate_turn, TurnOutput};
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::{
    PendingGeneratedMaterials, SignalNetworkCache, SimulationStepStats,
};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::simulation::SimulationWorlds;
use crate::game::world::block_instance::MaterialBlockRegistry;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;

use super::control::SimulationControl;
use super::SimulationDebugLog;

pub struct IntrospectionContext<'a> {
    pub world: &'a WorldBlocks,
    pub turn_structures: &'a StructureState,
    pub solution_structures: StructureState,
    pub factory_registry: &'a FactoryBlockRegistry,
    pub control: &'a SimulationControl,
    pub signal_cache: &'a mut SignalNetworkCache,
    pub pusher_state: &'a PusherState,
    pub movement_influence: &'a mut MovementInfluenceCache,
}

pub struct SimCoreWorld<'w> {
    world: &'w mut World,
}

impl<'w> SimCoreWorld<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self { world }
    }

    pub fn control(&self) -> &SimulationControl {
        self.world.resource()
    }

    pub fn world_blocks(&self) -> &WorldBlocks {
        self.world.resource()
    }

    pub fn world_blocks_mut(&mut self) -> Mut<'_, WorldBlocks> {
        self.world.resource_mut()
    }

    pub fn pusher_state(&self) -> &PusherState {
        self.world.resource()
    }

    pub fn world(&self) -> &World {
        self.world
    }

    pub fn world_scope<R>(
        &mut self,
        f: impl FnOnce(&WorldBlocks, &mut SignalNetworkCache) -> R,
    ) -> R {
        self.world
            .resource_scope(|world, mut signal_cache: Mut<SignalNetworkCache>| {
                let blocks = world.resource::<WorldBlocks>();
                f(blocks, &mut signal_cache)
            })
    }

    pub fn introspection_scope<R>(&mut self, f: impl FnOnce(IntrospectionContext<'_>) -> R) -> R {
        self.world
            .resource_scope(|world, mut signal_cache: Mut<SignalNetworkCache>| {
                world.resource_scope(
                    |world, mut movement_influence: Mut<MovementInfluenceCache>| {
                        let control = world.resource::<SimulationControl>();
                        let blocks = world.resource::<WorldBlocks>();
                        let turn_structures = world.resource::<StructureState>();
                        let pusher_state = world.resource::<PusherState>();
                        let factory_registry = world.resource::<FactoryBlockRegistry>();
                        let solution_structures = control
                            .start_structures
                            .clone()
                            .unwrap_or_else(|| turn_structures.clone());
                        f(IntrospectionContext {
                            world: blocks,
                            turn_structures,
                            solution_structures,
                            factory_registry,
                            control,
                            signal_cache: &mut signal_cache,
                            pusher_state,
                            movement_influence: &mut movement_influence,
                        })
                    },
                )
            })
    }

    pub fn is_active(&self) -> bool {
        self.control().is_active()
    }

    pub fn begin_simulation(&mut self) {
        if self.is_active() {
            return;
        }
        self.world
            .resource_scope(|world, mut control: Mut<SimulationControl>| {
                let world_blocks = world.resource::<WorldBlocks>().clone();
                control.start_snapshot = Some(world_blocks.clone());
                *world.resource_mut::<PusherState>() =
                    PusherState::rebuild_from_world(&world_blocks);
                world
                    .resource_mut::<StructureState>()
                    .rebuild_for_simulation(&world_blocks);
                control.start_structures = Some(world.resource::<StructureState>().clone());
                let mut factory_registry = world.resource_mut::<FactoryBlockRegistry>();
                *factory_registry = FactoryBlockRegistry::rebuild_from_world(&world_blocks);
                factory_registry.freeze_solution();
                *world.resource_mut::<MaterialBlockRegistry>() =
                    MaterialBlockRegistry::rebuild_from_world(&world_blocks);
            });
    }

    pub fn request_continuous_run(&mut self) {
        self.begin_simulation();
        self.world.resource_mut::<SimulationControl>().running = true;
    }

    pub fn request_one_turn(&mut self) -> Result<(), &'static str> {
        if !self.is_active() {
            return Err("simulation is not active");
        }
        let mut control = self.world.resource_mut::<SimulationControl>();
        control.running = false;
        control.speed = 1.0;
        control.step_requested = true;
        Ok(())
    }

    pub fn rollback(&mut self) -> Option<StructureState> {
        self.world
            .resource_scope(|world, mut control: Mut<SimulationControl>| {
                control.running = false;
                control.step_requested = false;
                control.turn = 0;
                control.accumulator = 0.0;
                world.resource_mut::<PendingGeneratedMaterials>().clear();
                world.resource_mut::<MovementInfluenceCache>().clear();
                world.resource_mut::<PusherState>().clear();
                world.resource_mut::<FactoryBlockRegistry>().clear();
                world.resource_mut::<MaterialBlockRegistry>().clear();
                let factory_snapshot = control.start_structures.take();
                if let Some(snapshot) = control.start_snapshot.take() {
                    *world.resource_mut::<WorldBlocks>() = snapshot;
                } else {
                    let mut blocks = world.resource_mut::<WorldBlocks>();
                    blocks.retain(|_, block| !block.kind.is_material());
                    blocks.clear_generated_markers();
                }
                factory_snapshot
            })
    }

    pub fn reset(&mut self) {
        self.rollback();
        *self.world.resource_mut::<WorldBlocks>() = WorldBlocks::default();
        self.world
            .resource_mut::<PendingGeneratedMaterials>()
            .clear();
        *self.world.resource_mut::<SignalNetworkCache>() = SignalNetworkCache::default();
        self.world.resource_mut::<StructureState>().clear();
        self.world.resource_mut::<MovementInfluenceCache>().clear();
        self.world.resource_mut::<PusherState>().clear();
        let mut control = self.world.resource_mut::<SimulationControl>();
        control.turn = 0;
        control.running = false;
        control.step_requested = false;
        control.accumulator = 0.0;
        control.speed = 1.0;
    }

    pub fn simulate_next_turn(
        &mut self,
        _animation_duration: f32,
        sim_log: Option<&mut SimulationDebugLog>,
        stats: Option<&mut SimulationStepStats>,
    ) -> TurnOutput {
        let next_turn = self.world.resource::<SimulationControl>().turn + 1;
        let mut state = SystemState::<(
            ResMut<'static, SimulationControl>,
            ResMut<'static, WorldBlocks>,
            ResMut<'static, PendingGeneratedMaterials>,
            ResMut<'static, SignalNetworkCache>,
            ResMut<'static, StructureState>,
            ResMut<'static, FactoryBlockRegistry>,
            ResMut<'static, MaterialBlockRegistry>,
            ResMut<'static, MovementInfluenceCache>,
            ResMut<'static, PusherState>,
        )>::new(self.world);
        let (
            mut control,
            mut world_blocks,
            mut pending_generated,
            mut signal_cache,
            mut structure_state,
            mut factory_registry,
            mut material_registry,
            mut movement_influence,
            mut pusher_state,
        ) = state.get_mut(self.world);
        let solution = control
            .start_snapshot
            .clone()
            .unwrap_or_else(|| world_blocks.clone());
        let solution_structures = control
            .start_structures
            .clone()
            .unwrap_or_else(|| structure_state.clone());
        let mut worlds = SimulationWorlds::from_snapshot_parts(
            solution,
            solution_structures,
            world_blocks.clone(),
            structure_state.clone(),
            factory_registry.clone(),
            material_registry.clone(),
        );
        let output = simulate_turn(
            &mut worlds,
            &mut pending_generated,
            &mut signal_cache,
            next_turn,
            &mut pusher_state,
            &mut movement_influence,
            sim_log,
            stats,
        );
        *world_blocks = worlds.turn;
        *structure_state = worlds.turn_structures;
        *factory_registry = worlds.factory_registry;
        *material_registry = worlds.material_registry;
        control.turn = next_turn;
        output
    }
}
