use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game::blocks::{BlockModel, BlockModelPart, ModelMesh};
use crate::game::world::animation::{AnimatedPusher, AnimatedPusherRod, PusherAnimation};
use crate::game::world::render_assets::WorldRenderAssets;
use crate::game::world::rendering::BlockIconRenderEntity;

pub fn spawn_model_parts(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    model: BlockModel,
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    match model {
        BlockModel::Default => {}
        BlockModel::Parts(parts) | BlockModel::PartsOnly(parts) => {
            for &part in parts {
                spawn_static_part(parent, assets, part, icon_layer, preview);
            }
        }
        BlockModel::PusherParts(parts) => {
            spawn_pusher_model_parts(
                parent,
                assets,
                parts,
                pusher_animation,
                icon_layer,
                preview,
            );
        }
    }
}

fn spawn_pusher_model_parts(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    parts: &'static [BlockModelPart],
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    use crate::game::blocks::pusher::model::{
        pusher_rod_center_z, pusher_rod_length, ROD_BASE_LENGTH,
    };

    for part in parts {
        let mut translation = model_vec3(part.translation);
        let mut scale = model_vec3(part.scale);

        if part.mesh == ModelMesh::PusherHead {
            if let Some(animation) = pusher_animation {
                translation += Vec3::NEG_Z * animation.to_extension;
            }
        } else if part.mesh == ModelMesh::RodZ {
            let extension = pusher_animation
                .map(|animation| animation.to_extension)
                .unwrap_or(0.0);
            let length = pusher_rod_length(extension);
            translation.z = pusher_rod_center_z(extension);
            scale = Vec3::new(scale.x, scale.y, length / ROD_BASE_LENGTH);
        }

        spawn_part_mesh(
            parent,
            assets,
            *part,
            translation,
            scale,
            icon_layer,
            preview,
            pusher_animation,
        );
    }
}

fn spawn_static_part(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    part: BlockModelPart,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    let translation = model_vec3(part.translation);
    let scale = model_vec3(part.scale);
    spawn_part_mesh(
        parent,
        assets,
        part,
        translation,
        scale,
        icon_layer,
        preview,
        None,
    );
}

fn spawn_part_mesh(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    part: BlockModelPart,
    translation: Vec3,
    scale: Vec3,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
    pusher_animation: Option<PusherAnimation>,
) {
    let mut child = parent.spawn((
        Mesh3d(assets.model_mesh(part.mesh)),
        MeshMaterial3d(if preview {
            assets.model_preview_material(part.material)
        } else {
            assets.model_material(part.material)
        }),
        Transform {
            translation,
            rotation: Quat::from_rotation_y(part.yaw_radians),
            scale,
            ..default()
        },
    ));
    if let Some(icon_layer) = icon_layer {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    if let Some(animation) = pusher_animation.filter(|animation| {
        animation.duration > 0.0 && animation.from_extension != animation.to_extension
    }) {
        if part.mesh == ModelMesh::PusherHead {
            child.insert(AnimatedPusher::new(
                animation,
                model_vec3(part.translation),
            ));
        } else if part.mesh == ModelMesh::RodZ {
            child.insert(AnimatedPusherRod::new(animation, model_vec3(part.scale)));
        }
    }
}

fn model_vec3(value: [f32; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}
