use super::DrillHeadBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MaterialDestroyer};
use crate::world::direction::{Facing};

impl BlockBehavior for DrillHeadBlock {
    fn material_destroyer(&self, _facing: Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::AdjacentDrillHead)
    }
}
