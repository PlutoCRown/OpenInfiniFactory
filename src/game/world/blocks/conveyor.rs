use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct ConveyorBlock;

pub static CONVEYOR: ConveyorBlock = ConveyorBlock;

impl Block for ConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Conveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.conveyor",
            "short.conveyor",
            rgb(0.10, 0.22, 0.28),
            rgb(0.08, 0.20, 0.26),
        )
        .directional()
        .alternate(BlockKind::ReverseConveyor)
    }

    fn conveyor_source_offset(&self) -> Option<IVec3> {
        Some(IVec3::Y)
    }
}

impl FactoryBlock for ConveyorBlock {}
