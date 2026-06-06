use super::GeneratorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MaterialSource};
use crate::game::world::direction::{Facing};

impl BlockBehavior for GeneratorBlock {
    fn material_source(&self, facing: Facing) -> Option<MaterialSource> {
        let _ = facing;
        Some(MaterialSource::Generator)
    }
}
