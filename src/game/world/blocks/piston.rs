use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct PistonBlock;

pub static PISTON: PistonBlock = PistonBlock;

impl Block for PistonBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Piston
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.piston",
            "short.piston",
            rgb(0.78, 0.55, 0.28),
            rgb(0.66, 0.43, 0.20),
        )
        .directional()
        .alternate(BlockKind::Blocker)
    }

    fn is_powered_device(&self) -> bool {
        true
    }

    fn is_piston(&self) -> bool {
        true
    }

    fn blocks_wire_connector(&self) -> bool {
        true
    }
}

impl FactoryBlock for PistonBlock {}
