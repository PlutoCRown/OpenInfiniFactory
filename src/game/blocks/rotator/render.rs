use super::RotatorBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, render_bottom_wire_device};
use crate::game::world::direction::Facing;

impl BlockRender for RotatorBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        render_bottom_wire_device()
    }
}
