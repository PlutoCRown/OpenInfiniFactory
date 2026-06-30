use bevy::prelude::*;

use crate::game::state::UiPanelId;
use crate::game::world::direction::Facing;
use crate::game::world::grid::BlockSettings;

use super::traits::{BlockBehavior, BlockMeta, BlockRender, BlockUi, PlaceableBlock};
use super::{
    Block, BlockDefinition, BlockKind, BlockModel, EditableBlock, MarkerBehavior,
    MaterialDestroyer, MaterialKind, MaterialLabeler, MaterialSource, MovementRule,
    PersistentLayer, RenderBehavior, SignalBehavior, WeldBehavior,
};

/// Wraps a block type so sub-trait impls in separate files compose into [`Block`].
pub struct BlockImpl<T>(pub T);

impl<T> Block for BlockImpl<T>
where
    T: BlockMeta + BlockBehavior + BlockRender + Send + Sync,
{
    fn id(&self) -> BlockKind {
        self.0.id()
    }

    fn definition(&self) -> BlockDefinition {
        self.0.definition()
    }

    fn is_directional(&self) -> bool {
        self.0.is_directional()
    }

    fn non_connection_face(&self, facing: Facing) -> Option<IVec3> {
        self.0.non_connection_face(facing)
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        self.0.marker_behavior(facing)
    }

    fn material_source(&self, facing: Facing) -> Option<MaterialSource> {
        self.0.material_source(facing)
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        self.0.material_kind()
    }

    fn persistent_layer(&self) -> Option<PersistentLayer> {
        self.0.persistent_layer()
    }

    fn default_settings(&self, pos: IVec3) -> Option<BlockSettings> {
        self.0.default_settings(pos)
    }

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        self.0.movement_rule(facing)
    }

    fn material_destroyer(&self, facing: Facing) -> Option<MaterialDestroyer> {
        self.0.material_destroyer(facing)
    }

    fn material_labeler(&self, facing: Facing) -> Option<MaterialLabeler> {
        self.0.material_labeler(facing)
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        self.0.weld_behavior()
    }

    fn signal_behavior(&self, facing: Facing) -> Option<SignalBehavior> {
        self.0.signal_behavior(facing)
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        self.0.render_behavior(facing)
    }

    fn model(&self) -> BlockModel {
        self.0.model()
    }

    fn block_texture(&self) -> Option<Image> {
        self.0.block_texture()
    }

    fn alternate(&self) -> Option<BlockKind> {
        self.0.alternate()
    }
}

impl<T> EditableBlock for BlockImpl<T>
where
    T: BlockMeta + BlockBehavior + BlockRender + BlockUi + Send + Sync,
{
    fn ui_panel(&self) -> Option<UiPanelId> {
        self.0.ui_panel()
    }
}

impl<T> PlaceableBlock for BlockImpl<T>
where
    T: PlaceableBlock + Send + Sync,
{
    fn item_slot_color(&self) -> Color {
        self.0.item_slot_color()
    }
}
