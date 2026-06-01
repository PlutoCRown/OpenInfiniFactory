use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn render_behavior(_block: &WireBlock, _facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Wire),
        ..Default::default()
    }
}
