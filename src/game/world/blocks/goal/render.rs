use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::Plate,
        ModelMeshSpec::Cuboid {
            size: [0.78, 0.06, 0.78],
        },
    ),
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
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[(
    ModelMaterial::Goal,
    ModelMaterialSpec::Emissive {
        color: super::rgb(0.55, 1.0, 0.36),
        emissive: super::rgb(0.05, 0.22, 0.04),
    },
)];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &GoalBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Goal, [0.0, 0.18, 0.0]),
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Goal, [0.0, 0.44, 0.0])
        .scaled([0.74, 0.74, 0.74]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Goal, [0.0, 0.66, 0.0]),
];

pub(super) fn render_behavior(_block: &GoalBlock, _facing: Facing) -> RenderBehavior {
    RenderBehavior {
        goal_topper: true,
        ..Default::default()
    }
}

pub(super) fn model(_block: &GoalBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
