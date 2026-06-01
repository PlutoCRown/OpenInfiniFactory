use super::*;
use crate::game::world::blocks::*;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::Large,
        ModelMeshSpec::Cuboid {
            size: [0.72, 0.22, 0.72],
        },
    ),
    (
        ModelMesh::RodX,
        ModelMeshSpec::Cuboid {
            size: [0.72, 0.12, 0.12],
        },
    ),
    (
        ModelMesh::Small,
        ModelMeshSpec::Cuboid {
            size: [0.22, 0.22, 0.22],
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
        ModelMaterial::Signal,
        ModelMaterialSpec::Emissive {
            color: super::rgb(0.12, 0.78, 1.0),
            emissive: super::rgb(0.02, 0.18, 0.24),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &RollerBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::System, [0.0, 0.38, 0.04]),
    BlockModelPart::new(ModelMesh::RodX, ModelMaterial::Signal, [0.0, 0.38, -0.40])
        .scaled([0.82, 0.82, 0.82]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::Signal,
        [-0.42, 0.38, -0.40],
    ),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Signal, [0.42, 0.38, -0.40]),
];

pub(super) fn model(_block: &RollerBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
