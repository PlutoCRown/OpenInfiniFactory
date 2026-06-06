use super::DownWelderBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, RenderBehavior, WeldConnectorBehavior};
use bevy::prelude::{IVec3};
use crate::game::world::direction::{Facing};

const MODEL: &[crate::game::blocks::BlockModelPart] = &[];

impl BlockRender for DownWelderBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(IVec3::NEG_Y)),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
