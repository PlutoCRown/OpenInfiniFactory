use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

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
        .directional()
        .alternate(BlockKind::Conveyor)
    }

    fn conveyor_source_offset(&self) -> Option<IVec3> {
        Some(IVec3::NEG_Y)
    }
}

impl FactoryBlock for ReverseConveyorBlock {}
