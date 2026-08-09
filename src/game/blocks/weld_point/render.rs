use super::WeldPointBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, WeldConnectorBehavior};
use crate::game::world::direction::Facing;

impl BlockRender for WeldPointBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::AllSides),
            ..Default::default()
        }
    }
}
