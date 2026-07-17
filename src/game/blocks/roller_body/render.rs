use super::RollerBodyBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::BlockModel;

impl BlockRender for RollerBodyBlock {
    fn model(&self) -> BlockModel {
        // 无模型：视觉仍由宿主 Roller 承担
        BlockModel::PartsOnly(&[])
    }
}
