use bevy::prelude::*;

use crate::game::state::UiPanelId;
use crate::game::world::direction::Facing;
use crate::game::world::grid::BlockSettings;

use oif_sim::blocks::traits::{BlockBehavior, BlockMeta};
use oif_sim::blocks::{
    BlockDefinition, BlockKind, LaserOpticsBehavior, MarkerBehavior, MaterialDestroyer,
    MaterialLabeler, MaterialProcessor, MaterialSource, MovementRule, PersistentLayer,
    SignalBehavior, WeldBehavior,
};

use super::traits::{BlockRender, BlockUi, PlaceableBlock};
use super::{BlockModel, RenderBehavior};

/// 包装各方块类型，使分文件的 sub-trait impl 能注册进 inventory
pub struct BlockImpl<T>(pub T);

impl<T> BlockMeta for BlockImpl<T>
where
    T: BlockMeta + Send + Sync,
{
    fn id(&self) -> BlockKind {
        self.0.id()
    }

    fn definition(&self) -> BlockDefinition {
        self.0.definition()
    }

    fn alternate(&self) -> Option<BlockKind> {
        self.0.alternate()
    }

    fn persistent_layer(&self) -> Option<PersistentLayer> {
        self.0.persistent_layer()
    }

    fn default_settings(&self, pos: IVec3) -> Option<BlockSettings> {
        self.0.default_settings(pos)
    }
}

impl<T> BlockBehavior for BlockImpl<T>
where
    T: BlockBehavior + Send + Sync,
{
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

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        self.0.movement_rule(facing)
    }

    fn material_destroyer(&self, facing: Facing) -> Option<MaterialDestroyer> {
        self.0.material_destroyer(facing)
    }

    fn material_labeler(&self, facing: Facing) -> Option<MaterialLabeler> {
        self.0.material_labeler(facing)
    }

    fn material_processor(&self) -> Option<MaterialProcessor> {
        self.0.material_processor()
    }

    fn laser_optics(&self) -> Option<LaserOpticsBehavior> {
        self.0.laser_optics()
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        self.0.weld_behavior()
    }

    fn signal_behavior(&self, facing: Facing) -> Option<SignalBehavior> {
        self.0.signal_behavior(facing)
    }
}

impl<T> BlockRender for BlockImpl<T>
where
    T: BlockRender + Send + Sync,
{
    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        self.0.render_behavior(facing)
    }

    fn model(&self) -> BlockModel {
        self.0.model()
    }

    fn block_texture(&self) -> Option<Image> {
        self.0.block_texture()
    }
}

impl<T> BlockUi for BlockImpl<T>
where
    T: BlockUi + Send + Sync,
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
