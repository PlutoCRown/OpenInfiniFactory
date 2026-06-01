use super::*;
use crate::game::world::blocks::*;

pub(super) fn model(_block: &DrillHeadBlock) -> BlockModel {
    BlockModel::PartsOnly(&[])
}
