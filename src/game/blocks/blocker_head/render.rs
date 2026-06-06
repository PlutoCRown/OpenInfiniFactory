use super::BlockerHeadBlock;

use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::{BlockModel};

impl BlockRender for BlockerHeadBlock {
    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(&[])
    }
}
