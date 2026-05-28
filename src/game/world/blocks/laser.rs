use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct LaserBlock;

pub static LASER: LaserBlock = LaserBlock;

impl Block for LaserBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Laser
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.laser",
            "short.laser",
            rgb(0.85, 0.20, 0.34),
            rgb(0.72, 0.12, 0.26),
        )
        .directional()
        .alternate(BlockKind::Drill)
    }

    fn is_powered_device(&self) -> bool {
        true
    }

    fn is_laser(&self) -> bool {
        true
    }

    fn blocks_wire_connector(&self) -> bool {
        true
    }
}

impl FactoryBlock for LaserBlock {}
