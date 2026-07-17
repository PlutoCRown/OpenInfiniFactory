use super::StamperBodyBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::BlockModel;

impl BlockRender for StamperBodyBlock {
    fn model(&self) -> BlockModel {
        // 无模型：视觉仍由宿主 Stamper 承担
        BlockModel::PartsOnly(&[])
    }
}
