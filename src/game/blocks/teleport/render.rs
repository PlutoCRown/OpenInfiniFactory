use super::TeleportBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::BlockModel;

impl BlockRender for TeleportBlock {
    fn model(&self) -> BlockModel {
        // 视觉由 visual::spawn_teleport_visual + PortalMaterial 绘制
        BlockModel::PartsOnly(&[])
    }
}
