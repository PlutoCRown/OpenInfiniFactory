use super::*;
use crate::game::world::blocks::*;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::ConveyorBase,
        ModelMeshSpec::Cuboid {
            size: [1.0, 0.90, 1.0],
        },
    ),
    (
        ModelMesh::ConveyorBelt,
        ModelMeshSpec::Cuboid {
            size: [0.90, 0.10, 1.0],
        },
    ),
    (
        ModelMesh::RodX,
        ModelMeshSpec::Cuboid {
            size: [0.72, 0.12, 0.12],
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[
    (
        ModelMaterial::Belt,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.86, 0.46, 0.14),
        },
    ),
    (
        ModelMaterial::ConveyorBelt,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.02, 0.02, 0.02),
        },
    ),
    (
        ModelMaterial::BeltStripe,
        ModelMaterialSpec::Emissive {
            color: super::rgb(1.0, 0.76, 0.28),
            emissive: super::rgb(0.18, 0.10, 0.02),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &ConveyorBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::ConveyorBase,
        ModelMaterial::Belt,
        [0.0, 0.0, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::ConveyorBelt,
        ModelMaterial::ConveyorBelt,
        [0.0, 0.50, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [-0.11, 0.56, -0.26],
    )
    .scaled([0.62, 0.16, 0.42])
    .yawed(0.7853982),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::BeltStripe,
        [0.11, 0.56, -0.26],
    )
    .scaled([0.62, 0.16, 0.42])
    .yawed(-0.7853982),
];

pub(super) fn model(_block: &ConveyorBlock) -> BlockModel {
    BlockModel::PartsOnly(MODEL)
}
