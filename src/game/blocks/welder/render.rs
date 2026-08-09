use super::WelderBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, WeldConnectorBehavior};
use crate::game::world::direction::Facing;

impl BlockRender for WelderBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(facing.forward_ivec3())),
            ..Default::default()
        }
    }
}
