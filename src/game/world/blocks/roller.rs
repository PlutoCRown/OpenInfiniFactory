use super::{
    edit_labeler, rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel,
    BlockModelPart, EditableBlock, MaterialLabeler, ModelMaterial, ModelMesh,
};
use crate::game::ui::{BlockEditAction, UiPanelId};
use crate::game::world::grid::{BlockSettings, LabelerSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::System, [0.0, 0.38, 0.04]),
    BlockModelPart::new(ModelMesh::RodX, ModelMaterial::Signal, [0.0, 0.38, -0.40])
        .scaled([0.82, 0.82, 0.82]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Signal,
        [-0.42, 0.38, -0.40],
    ),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Signal, [0.42, 0.38, -0.40]),
];

pub struct RollerBlock;

pub static ROLLER: RollerBlock = RollerBlock;

impl Block for RollerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Roller
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
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

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Labeler(LabelerSettings::default()))
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
impl EditableBlock for RollerBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Labeler)
    }

    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        edit_labeler(ctx, action);
    }
}
