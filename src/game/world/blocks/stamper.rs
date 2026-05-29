use super::{
    rgb, Block, BlockDefinition, BlockKind, EditableBlock, MaterialLabeler, SystemBlock,
};
use crate::game::world::grid::{BlockSettings, LabelerSettings};

pub struct StamperBlock;

pub static STAMPER: StamperBlock = StamperBlock;

impl Block for StamperBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Stamper
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.stamper",
            "short.stamper",
            rgb(0.82, 0.26, 0.58),
            rgb(0.64, 0.14, 0.42),
        )
        .no_collision()
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn material_labeler(&self, facing: super::Facing) -> Option<MaterialLabeler> {
        Some(MaterialLabeler::Stamper {
            target: facing.forward_ivec3(),
        })
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Labeler(LabelerSettings::default()))
    }
}

impl SystemBlock for StamperBlock {}
impl EditableBlock for StamperBlock {}
