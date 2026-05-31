use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, EditableBlock,
    ModelMaterial, ModelMesh, SystemBlock,
};
use crate::game::ui::UiPanelId;
use crate::game::world::grid::{BlockSettings, TeleportSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::TeleportIn,
        [0.0, 0.22, -0.30],
    )
    .scaled([0.88, 0.72, 0.72]),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::TeleportIn,
        [0.0, 0.22, 0.30],
    )
    .scaled([0.88, 0.72, 0.72]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::TeleportIn,
        [-0.30, 0.22, 0.0],
    )
    .scaled([0.72, 0.72, 0.88]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::TeleportIn,
        [0.30, 0.22, 0.0],
    )
    .scaled([0.72, 0.72, 0.88]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportIn,
        [0.0, 0.42, 0.0],
    )
    .scaled([0.88, 0.88, 0.88]),
];

pub struct TeleportEntranceBlock;

pub static TELEPORT_ENTRANCE: TeleportEntranceBlock = TeleportEntranceBlock;

impl Block for TeleportEntranceBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportEntrance
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.teleport_entrance",
            "short.teleport_entrance",
            rgb(0.12, 0.62, 0.92),
            rgb(0.06, 0.42, 0.72),
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

impl SystemBlock for TeleportEntranceBlock {}
impl EditableBlock for TeleportEntranceBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Teleport)
    }
}
