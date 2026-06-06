use crate::game::blocks::{BlockModelPart, ModelMaterial, ModelMesh};

pub const BODY_BACK_Z: f32 = -0.30;
pub const HEAD_CENTER_Z: f32 = -0.40;
pub const HEAD_HALF_DEPTH: f32 = 0.10;
pub const ROD_BASE_LENGTH: f32 = 0.72;
pub const ROD_XY_SCALE: f32 = 1.35 * 3.0;
/// Recess the rod anchor toward the head side so it stays behind the back panel.
pub const ROD_PANEL_INSET: f32 = 0.10;

pub const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::PusherBody,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, 0.10],
    ),
    BlockModelPart::new(
        ModelMesh::PusherHead,
        ModelMaterial::BorderedWoodTexture,
        [0.0, 0.0, HEAD_CENTER_Z],
    ),
    BlockModelPart::new(
        ModelMesh::RodZ,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, 0.0],
    )
    .scaled([ROD_XY_SCALE, ROD_XY_SCALE, 1.0]),
];

pub fn pusher_rod_length(extension: f32) -> f32 {
    BODY_BACK_Z - (HEAD_CENTER_Z - HEAD_HALF_DEPTH - extension)
}

pub fn pusher_rod_center_z(extension: f32) -> f32 {
    let length = pusher_rod_length(extension);
    BODY_BACK_Z - length * 0.5 + ROD_PANEL_INSET
}
