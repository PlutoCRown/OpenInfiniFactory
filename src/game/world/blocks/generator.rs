use super::{rgb, Block, BlockDefinition, BlockKind, MaterialSource, SceneBlock};

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
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn material_source(&self, facing: super::Facing) -> Option<MaterialSource> {
        Some(MaterialSource::Generator {
            output: facing.forward_ivec3(),
        })
    }
}

impl SceneBlock for GeneratorBlock {}
