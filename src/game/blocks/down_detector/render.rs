use super::DownDetectorBlock;

use bevy::prelude::IVec3;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, render_directional_wire_device};
use crate::game::world::direction::Facing;

impl BlockRender for DownDetectorBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        render_directional_wire_device(IVec3::NEG_Y)
    }
}
