use super::GeneratorBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MaterialSource};
use crate::world::direction::{Facing};

impl BlockBehavior for GeneratorBlock {
    fn material_source(&self, facing: Facing) -> Option<MaterialSource> {
        let _ = facing;
        Some(MaterialSource::Generator)
    }

    fn shows_material_preview(&self) -> bool {
        true
    }
}
