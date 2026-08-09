use super::PusherBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{render_directional_wire_device, RenderBehavior};
use crate::game::world::direction::Facing;

impl BlockRender for PusherBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        render_directional_wire_device(facing.forward_ivec3())
    }
}
