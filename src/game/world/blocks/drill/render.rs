use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::DrillBody,
        ModelMeshSpec::Cuboid {
            size: [1.0, 1.0, 0.80],
        },
    ),
    (
        ModelMesh::DrillTip,
        ModelMeshSpec::DrillTip {
            radius: 0.34,
            length: 1.0,
            segments: 48,
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[
    (
        ModelMaterial::Platform,
        ModelMaterialSpec::Textured {
            color: super::rgb(1.0, 1.0, 1.0),
            texture: BlockTexture::Platform,
        },
    ),
    (
        ModelMaterial::DrillTip,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.82, 0.84, 0.82),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &DrillBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::DrillBody,
        ModelMaterial::Platform,
        [0.0, 0.0, 0.10],
    ),
    BlockModelPart::new(
        ModelMesh::DrillTip,
        ModelMaterial::DrillTip,
        [0.0, 0.0, -0.34],
    ),
];

pub(super) fn render_behavior(_block: &DrillBlock, facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Device {
            blocked_offset: facing.forward_ivec3(),
        }),
        ..Default::default()
    }
}

pub(super) fn model(_block: &DrillBlock) -> BlockModel {
    BlockModel::PartsOnly(MODEL)
}
