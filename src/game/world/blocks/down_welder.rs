use bevy::prelude::*;

use super::{rgb, Block, BlockData, BlockDefinition, BlockKind, Facing, FactoryBlock};

pub struct DownWelderBlock;

pub static DOWN_WELDER: DownWelderBlock = DownWelderBlock;

impl Block for DownWelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownWelder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_welder",
            "short.down_welder",
            rgb(0.92, 0.32, 0.20),
            rgb(0.78, 0.22, 0.14),
        )
        .alternate(BlockKind::Welder)
    }

    fn weld_marker(&self, _facing: Facing) -> Option<(IVec3, Facing)> {
        Some((IVec3::NEG_Y, Facing::North))
    }

    fn connects_to_weld_point(&self, _block: BlockData, connector_from_block: IVec3) -> bool {
        connector_from_block == IVec3::NEG_Y
    }
}

impl FactoryBlock for DownWelderBlock {}
