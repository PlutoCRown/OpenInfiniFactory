use super::WeldPointBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel, RenderBehavior, WeldConnectorBehavior};
use crate::game::world::direction::Facing;

impl BlockRender for WeldPointBlock {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::AllSides),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        // 只画连接杆，不画中心红点
        BlockModel::PartsOnly(&[])
    }
}
