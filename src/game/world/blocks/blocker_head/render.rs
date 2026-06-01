use super::*;
use crate::game::world::blocks::*;

pub(super) fn model(_block: &BlockerHeadBlock) -> BlockModel {
    BlockModel::PartsOnly(&[])
}
