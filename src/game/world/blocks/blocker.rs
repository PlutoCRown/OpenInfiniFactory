use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, Facing, FactoryBlock};

pub struct BlockerBlock;

pub static BLOCKER: BlockerBlock = BlockerBlock;

impl Block for BlockerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Blocker
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.blocker",
            "short.blocker",
            rgb(0.58, 0.40, 0.24),
            rgb(0.50, 0.32, 0.20),
        )
        .directional()
        .alternate(BlockKind::Piston)
    }

    fn blocker_marker(&self, facing: Facing) -> Option<(IVec3, Facing)> {
        Some((facing.forward_ivec3(), facing))
    }

    fn is_powered_device(&self) -> bool {
        true
    }

    fn blocks_wire_connector(&self) -> bool {
        true
    }
}

impl FactoryBlock for BlockerBlock {}
