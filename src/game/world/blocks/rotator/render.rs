use super::*;
use crate::game::world::blocks::*;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::RotatorBase,
        ModelMeshSpec::Cuboid {
            size: [1.0, 0.80, 1.0],
        },
    ),
    (
        ModelMesh::RotatorDisk,
        ModelMeshSpec::Cylinder {
            radius: 0.40,
            height: 0.20,
            resolution: 48,
        },
    ),
    (
        ModelMesh::RotatorRing,
        ModelMeshSpec::Ring {
            outer_radius: 0.50,
            inner_radius: 0.40,
            height: 0.20,
            segments: 64,
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[
    (
        ModelMaterial::PlatformBase,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.36, 0.47, 0.58),
        },
    ),
    (
        ModelMaterial::ConveyorBelt,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.02, 0.02, 0.02),
        },
    ),
    (
        ModelMaterial::Belt,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.86, 0.46, 0.14),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &RotatorBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::RotatorBase,
        ModelMaterial::PlatformBase,
        [0.0, 0.0, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RotatorDisk,
        ModelMaterial::ConveyorBelt,
        [0.0, 0.50, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RotatorRing,
        ModelMaterial::Belt,
        [0.0, 0.48, 0.0],
    ),
];

pub(super) fn model(_block: &RotatorBlock) -> BlockModel {
    BlockModel::PartsOnly(MODEL)
}
