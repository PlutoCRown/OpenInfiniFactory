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
    ModelMaterial::TeleportIn,
    ModelMaterialSpec::Emissive {
        color: super::rgb(0.18, 0.62, 1.0),
        emissive: super::rgb(0.02, 0.10, 0.34),
    },
)];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &TeleportEntranceBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

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

pub(super) fn model(_block: &TeleportEntranceBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
