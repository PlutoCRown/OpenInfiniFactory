use super::WireBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, WireConnectorBehavior};
use crate::game::world::direction::{Facing};

impl BlockRender for WireBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Wire),
            ..Default::default()
        }
    }
}
