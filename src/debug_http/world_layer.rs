use std::collections::HashMap;

use crate::game::world::grid::WorldBlocks;
use crate::sim_core::SimulationControl;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DebugWorldLayer {
    Turn,
    Solution,
}

pub fn parse_world_layer(params: &HashMap<String, String>) -> DebugWorldLayer {
    match params
        .get("world")
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("solution") => DebugWorldLayer::Solution,
        _ => DebugWorldLayer::Turn,
    }
}

pub fn parse_world_layer_option(world: Option<&str>) -> DebugWorldLayer {
    match world.map(|value| value.to_ascii_lowercase()).as_deref() {
        Some("solution") => DebugWorldLayer::Solution,
        _ => DebugWorldLayer::Turn,
    }
}

pub fn resolve_world_blocks<'a>(
    layer: DebugWorldLayer,
    turn: &'a WorldBlocks,
    control: &'a SimulationControl,
) -> Result<&'a WorldBlocks, String> {
    match layer {
        DebugWorldLayer::Turn => Ok(turn),
        DebugWorldLayer::Solution => control
            .start_snapshot
            .as_ref()
            .ok_or_else(|| "solution world requires an active simulation".to_string()),
    }
}

pub fn world_layer_label(layer: DebugWorldLayer) -> &'static str {
    match layer {
        DebugWorldLayer::Turn => "turn",
        DebugWorldLayer::Solution => "solution",
    }
}
