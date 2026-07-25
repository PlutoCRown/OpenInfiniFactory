use super::SignBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::BlockModel;

// 视觉由 visual::spawn_sign_visual 按墙贴/立杆生成
impl BlockRender for SignBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(&[])
    }
}
