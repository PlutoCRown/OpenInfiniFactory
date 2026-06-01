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

pub(super) fn assets(_block: &DetectorBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Signal, [0.0, 0.52, 0.0]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Signal, [0.0, 0.38, -0.34])
        .scaled([0.72, 0.72, 0.55]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.38, -0.52]),
];

pub(super) fn render_behavior(_block: &DetectorBlock, facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Device {
            blocked_offset: facing.forward_ivec3(),
        }),
        ..Default::default()
    }
}

pub(super) fn model(_block: &DetectorBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
