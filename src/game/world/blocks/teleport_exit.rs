use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, EditableBlock,
    ModelMaterial, ModelMesh,
};
use crate::game::ui::UiPanelId;
use crate::game::world::grid::{BlockSettings, TeleportSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::TeleportOut,
        [0.0, 0.22, -0.30],
    )
    .scaled([0.88, 0.72, 0.72]),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::TeleportOut,
        [0.0, 0.22, 0.30],
    )
    .scaled([0.88, 0.72, 0.72]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::TeleportOut,
        [-0.30, 0.22, 0.0],
    )
    .scaled([0.72, 0.72, 0.88]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::TeleportOut,
        [0.30, 0.22, 0.0],
    )
    .scaled([0.72, 0.72, 0.88]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportOut,
        [0.0, 0.42, 0.0],
    )
    .scaled([0.88, 0.88, 0.88]),
];

pub struct TeleportExitBlock;

pub static TELEPORT_EXIT: TeleportExitBlock = TeleportExitBlock;

impl Block for TeleportExitBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportExit
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.teleport_exit",
            "short.teleport_exit",
            rgb(0.72, 0.34, 0.96),
            rgb(0.50, 0.20, 0.74),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
impl EditableBlock for TeleportExitBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Teleport)
    }
}
