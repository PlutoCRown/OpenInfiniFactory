use super::LaserBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, render_directional_wire_device};
use crate::game::world::direction::Facing;

impl BlockRender for LaserBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        render_directional_wire_device(facing.forward_ivec3())
    }
}
