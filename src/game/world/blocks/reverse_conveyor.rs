use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MaterialMover};

pub struct ReverseConveyorBlock;

pub static REVERSE_CONVEYOR: ReverseConveyorBlock = ReverseConveyorBlock;

impl Block for ReverseConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::ReverseConveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.reverse_conveyor",
            "short.reverse_conveyor",
            rgb(0.14, 0.30, 0.36),
            rgb(0.10, 0.26, 0.32),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn material_mover(&self, facing: super::Facing) -> Option<MaterialMover> {
        Some(MaterialMover::Conveyor {
            source: IVec3::NEG_Y,
            offset: -facing.forward_ivec3(),
        })
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Conveyor)
    }
}

impl FactoryBlock for ReverseConveyorBlock {}
