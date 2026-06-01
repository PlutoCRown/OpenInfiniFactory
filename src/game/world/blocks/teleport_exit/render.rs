use super::*;
use crate::game::world::blocks::*;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::RodX,
        ModelMeshSpec::Cuboid {
            size: [0.72, 0.12, 0.12],
        },
    ),
    (
        ModelMesh::RodZ,
        ModelMeshSpec::Cuboid {
            size: [0.12, 0.12, 0.72],
        },
    ),
    (
        ModelMesh::Small,
        ModelMeshSpec::Cuboid {
            size: [0.22, 0.22, 0.22],
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[(
    ModelMaterial::TeleportOut,
    ModelMaterialSpec::Emissive {
        color: super::rgb(1.0, 0.54, 0.18),
        emissive: super::rgb(0.34, 0.10, 0.02),
    },
)];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &TeleportExitBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

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

pub(super) fn model(_block: &TeleportExitBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
