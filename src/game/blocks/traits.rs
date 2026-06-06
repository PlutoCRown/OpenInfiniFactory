use bevy::prelude::*;

use crate::game::state::UiPanelId;
use crate::game::world::grid::BlockSettings;

use super::{
    BlockDefinition, BlockKind, BlockModel, MarkerBehavior, MaterialDestroyer, MaterialKind,
    MaterialLabeler, MaterialSource, MovementRule, PersistentLayer, RenderBehavior, SignalBehavior,
    WeldBehavior,
};
use crate::game::world::direction::Facing;

/// Identity, catalog metadata, and per-instance defaults.
pub trait BlockMeta: Send + Sync {
    fn id(&self) -> BlockKind;
    fn definition(&self) -> BlockDefinition;

    fn alternate(&self) -> Option<BlockKind> {
        None
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        None
    }

    fn persistent_layer(&self) -> Option<PersistentLayer> {
        self.definition().persistence
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        None
    }
}

/// Simulation-facing behavior: movement, signals, markers, etc.
pub trait BlockBehavior: Send + Sync {
    fn is_directional(&self) -> bool {
        false
    }

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        None
    }

    fn material_source(&self, _facing: Facing) -> Option<MaterialSource> {
        None
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        None
    }

    fn material_destroyer(&self, _facing: Facing) -> Option<MaterialDestroyer> {
        None
    }

    fn material_labeler(&self, _facing: Facing) -> Option<MaterialLabeler> {
        None
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        None
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        None
    }
}

/// 3D model parts and connector rendering hints.
pub trait BlockRender: Send + Sync {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior::default()
    }

    fn model(&self) -> BlockModel {
        BlockModel::Default
    }
}

/// In-game property panel editing.
pub trait BlockUi: Send + Sync {
    fn ui_panel(&self) -> Option<UiPanelId>;
}
