use super::WelderBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, RenderBehavior, WeldConnectorBehavior};
use crate::game::world::direction::{Facing};

const MODEL: &[crate::game::blocks::BlockModelPart] = &[];

impl BlockRender for WelderBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(facing.forward_ivec3())),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
