use bevy::prelude::*;

use super::{rgb, Block, BlockData, BlockDefinition, BlockKind, Facing, FactoryBlock};

pub struct WelderBlock;

pub static WELDER: WelderBlock = WelderBlock;

impl Block for WelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Welder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.welder",
            "short.welder",
            rgb(0.76, 0.18, 0.16),
            rgb(0.62, 0.12, 0.12),
        )
        .directional()
        .alternate(BlockKind::DownWelder)
    }

    fn weld_marker(&self, facing: Facing) -> Option<(IVec3, Facing)> {
        Some((facing.forward_ivec3(), facing))
    }

    fn connects_to_weld_point(&self, block: BlockData, connector_from_block: IVec3) -> bool {
        connector_from_block == block.facing.forward_ivec3()
    }
}

impl FactoryBlock for WelderBlock {}
