use super::DownWelderBlock;

use bevy::prelude::IVec3;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{RenderBehavior, WeldConnectorBehavior};
use crate::game::world::direction::Facing;

impl BlockRender for DownWelderBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(IVec3::NEG_Y)),
            ..Default::default()
        }
    }
}
