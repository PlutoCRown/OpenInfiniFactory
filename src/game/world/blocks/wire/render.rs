use super::*;
use crate::game::world::blocks::{
    six_way_connection_plan, BlockData, BlockKind, BlockModel, RenderBehavior,
    WireConnectorBehavior,
};
use crate::game::world::direction::Facing;
use crate::game::world::grid::WorldBlocks;
use bevy::prelude::IVec3;

pub(super) fn render_behavior(_block: &WireBlock, _facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Wire),
        ..Default::default()
    }
}

pub(super) fn model(_block: &WireBlock) -> BlockModel {
    BlockModel::PartsOnly(&[])
}

pub struct WireConnectorRenderPlan {
    pub local_connector_offsets: Vec<IVec3>,
    pub isolated_wire_node: bool,
}

pub fn wire_connector_render_plan(
    data: BlockData,
    pos: IVec3,
    world: &WorldBlocks,
) -> Option<WireConnectorRenderPlan> {
    data.kind.render_spec(data.facing).behavior.wire_connector?;

    let plan = six_way_connection_plan(data, pos, world, connects_to_wire);

    Some(WireConnectorRenderPlan {
        isolated_wire_node: data.kind == BlockKind::Wire && plan.local_offsets.is_empty(),
        local_connector_offsets: plan.local_offsets,
    })
}

fn connects_to_wire(block: &BlockData, wire_from_block: IVec3) -> bool {
    match block.kind.render_spec(block.facing).behavior.wire_connector {
        Some(WireConnectorBehavior::Wire) => true,
        Some(WireConnectorBehavior::Device { blocked_offset }) => wire_from_block != blocked_offset,
        None => false,
    }
}
