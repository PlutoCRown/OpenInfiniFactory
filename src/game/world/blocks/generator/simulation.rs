use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn material_source(_block: &GeneratorBlock, facing: Facing) -> Option<MaterialSource> {
    let _ = facing;
    Some(MaterialSource::Generator)
}
