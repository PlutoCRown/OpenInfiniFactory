use super::{
    rgb, Block, BlockDefinition, BlockKind, EditableBlock, MaterialLabeler, SystemBlock,
};

pub struct RollerBlock;

pub static ROLLER: RollerBlock = RollerBlock;

impl Block for RollerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Roller
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.roller",
            "short.roller",
            rgb(0.18, 0.62, 0.78),
            rgb(0.10, 0.44, 0.60),
        )
        .no_collision()
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn material_labeler(&self, facing: super::Facing) -> Option<MaterialLabeler> {
        Some(MaterialLabeler::Roller {
            target: facing.forward_ivec3(),
        })
    }
}

impl SystemBlock for RollerBlock {}
impl EditableBlock for RollerBlock {}
