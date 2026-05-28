use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, Facing, FactoryBlock};

pub struct DrillBlock;

pub static DRILL: DrillBlock = DrillBlock;

impl Block for DrillBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Drill
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.drill",
            "short.drill",
            rgb(0.32, 0.36, 0.40),
            rgb(0.24, 0.26, 0.30),
        )
        .directional()
        .alternate(BlockKind::Laser)
    }

    fn drill_marker(&self, facing: Facing) -> Option<(IVec3, Facing)> {
        Some((facing.forward_ivec3(), facing))
    }

    fn is_powered_device(&self) -> bool {
        true
    }

    fn is_drill(&self) -> bool {
        true
    }

    fn blocks_wire_connector(&self) -> bool {
        true
    }
}

impl FactoryBlock for DrillBlock {}
