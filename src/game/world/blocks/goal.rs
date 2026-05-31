use super::{
    rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel, BlockModelPart,
    EditableBlock, ModelMaterial, ModelMesh, RenderBehavior,
};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::grid::{BlockSettings, GoalSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Goal, [0.0, 0.18, 0.0]),
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Goal, [0.0, 0.44, 0.0])
        .scaled([0.74, 0.74, 0.74]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Goal, [0.0, 0.66, 0.0]),
];

pub struct GoalBlock;

pub static GOAL: GoalBlock = GoalBlock;

impl Block for GoalBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Goal
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.goal",
            "short.goal",
            rgb(0.35, 0.72, 0.42),
            rgb(0.24, 0.56, 0.30),
        )
        .no_collision()
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            goal_topper: true,
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Goal(GoalSettings::default()))
    }
}
impl EditableBlock for GoalBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Goal)
    }

    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        let mut settings = ctx.world.goal_settings(ctx.pos);
        match action {
            BlockEditAction::ToggleMaterialDropdown => {
                ctx.toggle_dropdown(BlockPanelDropdown::GoalMaterial);
                return;
            }
            BlockEditAction::SetMaterial(material) => {
                settings.material = material;
                ctx.close_dropdown();
            }
            _ => return,
        }
        ctx.world.set_goal_settings(ctx.pos, settings);
        ctx.mark_dirty();
    }
}
