use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const RENDER_MESHES: &[(ModelMesh, ModelMeshSpec)] = &[
    (
        ModelMesh::Medium,
        ModelMeshSpec::Cuboid {
            size: [0.44, 0.20, 0.44],
        },
    ),
    (
        ModelMesh::RodY,
        ModelMeshSpec::Cuboid {
            size: [0.12, 0.72, 0.12],
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
        ModelMaterial::Signal,
        ModelMaterialSpec::Emissive {
            color: super::rgb(0.12, 0.78, 1.0),
            emissive: super::rgb(0.02, 0.18, 0.24),
        },
    ),
    (
        ModelMaterial::Power,
        ModelMaterialSpec::Emissive {
            color: super::rgb(1.0, 0.52, 0.20),
            emissive: super::rgb(0.22, 0.08, 0.02),
        },
    ),
];

const RENDER_ASSETS: BlockRenderAssets = BlockRenderAssets {
    meshes: RENDER_MESHES,
    materials: RENDER_MATERIALS,
};

pub(super) fn assets(_block: &DownDetectorBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Signal, [0.0, 0.30, 0.0]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Signal, [0.0, -0.10, 0.0])
        .scaled([0.76, 0.62, 0.76]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, -0.50, 0.0]),
];

pub(super) fn render_behavior(_block: &DownDetectorBlock, _facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Device {
            blocked_offset: IVec3::NEG_Y,
        }),
        ..Default::default()
    }
}

pub(super) fn model(_block: &DownDetectorBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
