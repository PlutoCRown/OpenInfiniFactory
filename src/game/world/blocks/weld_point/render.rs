use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[(
    ModelMesh::Small,
    ModelMeshSpec::Cuboid {
        size: [0.22, 0.22, 0.22],
    },
)];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[(
    ModelMaterial::WeldCore,
    ModelMaterialSpec::Emissive {
        color: super::rgb(1.0, 0.22, 0.10),
        emissive: super::rgb(0.22, 0.04, 0.02),
    },
)];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &WeldPointBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::Small,
    ModelMaterial::WeldCore,
    [0.0, 0.0, 0.0],
)];

pub(super) fn render_behavior(_block: &WeldPointBlock, _facing: Facing) -> RenderBehavior {
    RenderBehavior {
        weld_connector: Some(WeldConnectorBehavior::AllSides),
        ..Default::default()
    }
}

pub(super) fn model(_block: &WeldPointBlock) -> BlockModel {
    BlockModel::PartsOnly(MODEL)
}
