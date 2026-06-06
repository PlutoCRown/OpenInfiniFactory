use super::DrillHeadBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel};

impl BlockRender for DrillHeadBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(&[])
    }
}
