use super::BlockerBlock;

use crate::game::blocks::pusher::model::MODEL;
use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, RenderBehavior, WireConnectorBehavior};
use crate::game::world::direction::{Facing};

impl BlockRender for BlockerBlock {
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Device {
                blocked_offset: facing.forward_ivec3(),
            }),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }
}
