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
    (
        ModelMesh::Plate,
        ModelMeshSpec::Cuboid {
            size: [0.78, 0.06, 0.78],
        },
    ),
];

const RENDER_MATERIALS: &[(ModelMaterial, ModelMaterialSpec)] = &[
    (
        ModelMaterial::DarkFrame,
        ModelMaterialSpec::Srgb {
            color: super::rgb(0.12, 0.13, 0.15),
        },
    ),
    (
        ModelMaterial::Laser,
        ModelMaterialSpec::Emissive {
            color: super::rgb(1.0, 0.10, 0.22),
            emissive: super::rgb(0.35, 0.01, 0.04),
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

pub(super) fn assets(_block: &LaserBlock) -> BlockRenderAssets {
    RENDER_ASSETS
}

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::Medium,
        ModelMaterial::DarkFrame,
        [0.0, 0.42, 0.08],
    ),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Laser, [0.0, 0.42, -0.30])
        .scaled([0.54, 0.54, 0.76]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Laser, [0.0, 0.42, -0.56]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Power, [0.0, 0.62, 0.08])
        .scaled([0.58, 0.58, 0.58]),
];

pub(super) fn render_behavior(_block: &LaserBlock, facing: Facing) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Device {
            blocked_offset: facing.forward_ivec3(),
        }),
        ..Default::default()
    }
}

pub(super) fn model(_block: &LaserBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
