use super::DrillHeadBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MaterialDestroyer};
use crate::game::world::direction::{Facing};

impl BlockBehavior for DrillHeadBlock {
    fn material_destroyer(&self, _facing: Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::AdjacentDrillHead)
    }
}
