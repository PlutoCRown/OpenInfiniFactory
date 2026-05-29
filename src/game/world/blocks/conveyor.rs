use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MaterialMover};

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

    fn material_mover(&self, facing: super::Facing) -> Option<MaterialMover> {
        Some(MaterialMover::Conveyor {
            source: IVec3::Y,
            offset: facing.forward_ivec3(),
        })
    }
}

impl FactoryBlock for ConveyorBlock {}
