use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::PusherBody,
        ModelMeshSpec::CoveredCuboid {
            size: [1.0, 1.0, 0.80],
        },
    ),
    (
        ModelMesh::PusherHead,
        ModelMeshSpec::CoveredCuboid {
            size: [1.0, 1.0, 0.20],
        },
    ),
    (
        ModelMesh::RodZ,
        ModelMeshSpec::Cuboid {
            size: [0.12, 0.12, 0.72],
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[
    (
        ModelMaterial::StoneTexture,
        ModelMaterialSpec::Textured {
            color: super::rgb(1.0, 1.0, 1.0),
            texture: BlockTexture::Stone,
        },
    ),
    (
        ModelMaterial::BorderedWoodTexture,
        ModelMaterialSpec::Textured {
            color: super::rgb(1.0, 1.0, 1.0),
            texture: BlockTexture::BorderedWood,
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &BlockerBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::PusherBody,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, 0.10],
    ),
    BlockModelPart::new(
        ModelMesh::PusherHead,
        ModelMaterial::BorderedWoodTexture,
        [0.0, 0.0, -0.40],
    ),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, -0.18],
    )
    .scaled([1.35, 1.35, 0.70]),
];

pub(super) fn render_behavior(_block: &BlockerBlock, facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Device {
            blocked_offset: facing.forward_ivec3(),
        }),
        ..Default::default()
    }
}

pub(super) fn model(_block: &BlockerBlock) -> BlockModel {
    BlockModel::PartsOnly(MODEL)
}
