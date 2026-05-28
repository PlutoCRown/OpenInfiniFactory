use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock};

pub struct DetectorBlock;

pub static DETECTOR: DetectorBlock = DetectorBlock;

impl Block for DetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Detector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.detector",
            "short.detector",
            rgb(0.15, 0.45, 0.72),
            rgb(0.12, 0.34, 0.62),
        )
        .directional()
    }

    fn is_detector(&self) -> bool {
        true
    }

    fn blocks_wire_connector(&self) -> bool {
        true
    }
}

impl FactoryBlock for DetectorBlock {}
