use super::*;
use crate::game::world::blocks::*;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::Plate,
        ModelMeshSpec::Cuboid {
            size: [0.78, 0.06, 0.78],
        },
    ),
    (
        ModelMesh::RodY,
        ModelMeshSpec::Cuboid {
            size: [0.12, 0.72, 0.12],
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[(
    ModelMaterial::Lift,
    ModelMaterialSpec::Emissive {
        color: super::rgb(0.35, 0.82, 1.0),
        emissive: super::rgb(0.03, 0.16, 0.22),
    },
)];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &LifterBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Lift, [0.0, 0.54, 0.0]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [-0.24, 0.18, -0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [0.24, 0.18, -0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [-0.24, 0.18, 0.24]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Lift, [0.24, 0.18, 0.24]),
];

pub(super) fn model(_block: &LifterBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
