use super::*;
use crate::game::world::blocks::*;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::Medium,
        ModelMeshSpec::Cuboid {
            size: [0.44, 0.20, 0.44],
        },
    ),
    (
        ModelMesh::Small,
        ModelMeshSpec::Cuboid {
            size: [0.22, 0.22, 0.22],
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
        ModelMaterial::System,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.35, 0.28, 0.48),
        },
    ),
    (
        ModelMaterial::TeleportIn,
        ModelMaterialSpec::Emissive {
            color: super::rgb(0.18, 0.62, 1.0),
            emissive: super::rgb(0.02, 0.10, 0.34),
        },
    ),
    (
        ModelMaterial::TeleportOut,
        ModelMaterialSpec::Emissive {
            color: super::rgb(1.0, 0.54, 0.18),
            emissive: super::rgb(0.34, 0.10, 0.02),
        },
    ),
    (
        ModelMaterial::SystemAccent,
        ModelMaterialSpec::Emissive {
            color: super::rgb(0.72, 0.58, 1.0),
            emissive: super::rgb(0.12, 0.08, 0.24),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &ConverterBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

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

pub(super) fn model(_block: &ConverterBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
