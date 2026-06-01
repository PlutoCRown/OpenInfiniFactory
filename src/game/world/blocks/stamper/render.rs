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
        ModelMesh::RodZ,
        ModelMeshSpec::Cuboid {
            size: [0.12, 0.12, 0.72],
        },
    ),
    (
        ModelMesh::Plate,
        ModelMeshSpec::Cuboid {
            size: [0.78, 0.06, 0.78],
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
        ModelMaterial::SystemAccent,
        ModelMaterialSpec::Emissive {
            color: super::rgb(0.72, 0.58, 1.0),
            emissive: super::rgb(0.12, 0.08, 0.24),
        },
    ),
    (
        ModelMaterial::Laser,
        ModelMaterialSpec::Emissive {
            color: super::rgb(1.0, 0.10, 0.22),
            emissive: super::rgb(0.35, 0.01, 0.04),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &StamperBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::System, [0.0, 0.38, 0.04]),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::SystemAccent,
        [0.0, 0.38, -0.30],
    )
    .scaled([0.56, 0.56, 0.58]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Laser, [0.0, 0.38, -0.54])
        .scaled([0.52, 0.70, 0.40]),
];

pub(super) fn model(_block: &StamperBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
