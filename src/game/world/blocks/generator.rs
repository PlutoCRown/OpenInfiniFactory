use super::{rgb, Block, BlockDefinition, BlockKind, SceneBlock};

pub struct GeneratorBlock;

pub static GENERATOR: GeneratorBlock = GeneratorBlock;

impl Block for GeneratorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Generator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::scene(
            self.id(),
            "block.generator",
            "short.generator",
            rgb(0.52, 0.30, 0.68),
            rgb(0.42, 0.20, 0.56),
        )
        .directional()
    }

    fn is_generator(&self) -> bool {
        true
    }
}

impl SceneBlock for GeneratorBlock {}
