use bevy::prelude::*;

use super::{rgb, rgba, Block, BlockData, BlockDefinition, BlockKind, SystemBlock};

pub struct WeldPointBlock;

pub static WELD_POINT: WeldPointBlock = WeldPointBlock;

impl Block for WeldPointBlock {
    fn id(&self) -> BlockKind {
        BlockKind::WeldPoint
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.weld_point",
            "short.weld_point",
            rgba(1.0, 0.28, 0.18, 0.45),
            rgb(0.86, 0.16, 0.12),
        )
        .node()
        .transparent()
        .no_collision()
    }

    fn is_weld_point(&self) -> bool {
        true
    }

    fn connects_to_weld_point(&self, _block: BlockData, _connector_from_block: IVec3) -> bool {
        true
    }
}

impl SystemBlock for WeldPointBlock {}
