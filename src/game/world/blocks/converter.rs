use super::{
    rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel, BlockModelPart,
    EditableBlock, ModelMaterial, ModelMesh,
};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::grid::{BlockSettings, ConverterMode, ConverterSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::System, [0.0, 0.38, 0.0]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportIn,
        [-0.28, 0.54, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportOut,
        [0.28, 0.54, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::SystemAccent,
        [0.0, 0.54, 0.0],
    )
    .scaled([0.62, 0.55, 0.55]),
];

pub struct ConverterBlock;

pub static CONVERTER: ConverterBlock = ConverterBlock;

impl Block for ConverterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Converter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.converter",
            "short.converter",
            rgb(0.50, 0.36, 0.78),
            rgb(0.36, 0.24, 0.62),
        )
        .no_collision()
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Converter(ConverterSettings::default()))
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
impl EditableBlock for ConverterBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Converter)
    }

    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        let mut settings = ctx.world.converter_settings(ctx.pos);
        match action {
            BlockEditAction::ToggleInputDropdown => {
                ctx.toggle_dropdown(BlockPanelDropdown::ConverterInput);
                return;
            }
            BlockEditAction::ToggleOutputDropdown => {
                ctx.toggle_dropdown(BlockPanelDropdown::ConverterOutput);
                return;
            }
            BlockEditAction::SetInput(material) => {
                settings.input = material;
                settings.mode = ConverterMode::SpecificInput;
                ctx.close_dropdown();
            }
            BlockEditAction::SetOutput(material) => {
                settings.output = material;
                ctx.close_dropdown();
            }
            _ => return,
        }
        ctx.world.set_converter_settings(ctx.pos, settings);
        ctx.mark_dirty();
    }
}
