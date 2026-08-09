use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::simulation::stats::SimulationStepStats;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::PortalFlashQueue;
use crate::sim_bridge::TurnOutput;

use super::incremental::apply_turn_output_incremental;
use super::scene_render::SceneRenderMut;

/// 将一回合输出落到场景实体（转发增量实现）
pub fn apply_turn_output(
    before: &WorldBlocks,
    after: &WorldBlocks,
    output: &TurnOutput,
    previous_powered_wires: &HashSet<IVec3>,
    animation_duration: f32,
    scene: &mut SceneRenderMut,
    stats: &mut SimulationStepStats,
    portal_flash_queue: &mut PortalFlashQueue,
) {
    apply_turn_output_incremental(
        before,
        after,
        output,
        previous_powered_wires,
        animation_duration,
        scene,
        stats,
        portal_flash_queue,
    );
}
