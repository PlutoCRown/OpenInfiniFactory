use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game::blocks::{BlockModel, BlockModelPart, ModelMesh};
use crate::game::world::animation::{
    AnimatedPusher, AnimatedPusherRod, PusherAnimation, SpinningDrillHead,
};
use crate::game::world::render_assets::{FactoryPartHandles, FactoryVisual, WorldRenderAssets};
use crate::game::world::rendering::BlockIconRenderEntity;

/// 生成方块模型零件（优先工厂 GLB，否则程序化）
pub fn spawn_model_parts(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    kind: crate::game::blocks::BlockKind,
    model: BlockModel,
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    if let Some(visual) = assets.factory_visual(kind) {
        match visual {
            FactoryVisual::Static {
                parts,
                local_rotation,
            } => {
                spawn_factory_static(parent, parts, *local_rotation, icon_layer, preview);
            }
            FactoryVisual::Drill { body, head } => {
                spawn_factory_drill(parent, body, head, icon_layer, preview);
            }
            FactoryVisual::Pusher { body, stage, head } => {
                spawn_factory_pusher(
                    parent,
                    body,
                    stage,
                    head,
                    pusher_animation,
                    icon_layer,
                    preview,
                );
            }
            // 电线在 spawn 连通逻辑里按面生成
            FactoryVisual::Wire { .. } => {}
        }
        return;
    }

    match model {
        BlockModel::Default => {}
        BlockModel::Parts(parts) | BlockModel::PartsOnly(parts) => {
            for &part in parts {
                spawn_static_part(parent, assets, part, icon_layer, preview);
            }
        }
        BlockModel::PusherParts(parts) => {
            spawn_pusher_model_parts(parent, assets, parts, pusher_animation, icon_layer, preview);
        }
    }
}

/// 生成静态工厂 GLB 零件（可选额外局部旋转）
fn spawn_factory_static(
    parent: &mut ChildSpawnerCommands,
    parts: &[FactoryPartHandles],
    local_rotation: Quat,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    if local_rotation == Quat::IDENTITY {
        for part in parts {
            spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
        }
        return;
    }
    let mut root = parent.spawn((
        Transform::from_rotation(local_rotation),
        Visibility::default(),
    ));
    if let Some(icon_layer) = icon_layer {
        root.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    root.with_children(|parent| {
        for part in parts {
            spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
        }
    });
}

/// 生成钻头 Body / Head；Head 挂持续旋转组件
fn spawn_factory_drill(
    parent: &mut ChildSpawnerCommands,
    body: &[FactoryPartHandles],
    head: &[FactoryPartHandles],
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    for part in body {
        spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
    }

    let mut head_root = parent.spawn((Transform::default(), Visibility::default()));
    if let Some(icon_layer) = icon_layer {
        head_root.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    if !preview && icon_layer.is_none() {
        head_root.insert(SpinningDrillHead::default());
    }
    head_root.with_children(|parent| {
        for part in head {
            spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
        }
    });
}

/// 生成活塞 Body / Stage / Head
fn spawn_factory_pusher(
    parent: &mut ChildSpawnerCommands,
    body: &[FactoryPartHandles],
    stage: &[FactoryPartHandles],
    head: &[FactoryPartHandles],
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    for part in body {
        spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
    }

    let extension = pusher_animation
        .map(|animation| animation.from_extension)
        .unwrap_or(0.0);
    let animate = pusher_animation.filter(|animation| {
        animation.duration.is_some_and(|duration| duration > 0.0)
            && animation.from_extension != animation.to_extension
    });

    let stage_translation = Vec3::NEG_Z * (extension * 0.5);
    let mut stage_root = parent.spawn((
        Transform::from_translation(stage_translation),
        Visibility::default(),
    ));
    if let Some(icon_layer) = icon_layer {
        stage_root.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    if let Some(animation) = animate {
        stage_root.insert(AnimatedPusher::with_factor(animation, Vec3::ZERO, 0.5));
    }
    stage_root.with_children(|parent| {
        for part in stage {
            spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
        }
    });

    let head_translation = Vec3::NEG_Z * extension;
    let mut head_root = parent.spawn((
        Transform::from_translation(head_translation),
        Visibility::default(),
    ));
    if let Some(icon_layer) = icon_layer {
        head_root.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    if let Some(animation) = animate {
        head_root.insert(AnimatedPusher::with_factor(animation, Vec3::ZERO, 1.0));
    }
    head_root.with_children(|parent| {
        for part in head {
            spawn_factory_part(parent, part, Transform::default(), icon_layer, preview);
        }
    });
}

/// 生成单段工厂 GLB 网格
fn spawn_factory_part(
    parent: &mut ChildSpawnerCommands,
    part: &FactoryPartHandles,
    transform: Transform,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    let mut child = parent.spawn((
        Mesh3d(part.mesh.clone()),
        MeshMaterial3d(if preview {
            part.preview_material.clone()
        } else {
            part.material.clone()
        }),
        transform,
    ));
    if let Some(icon_layer) = icon_layer {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
}

/// 按连通面生成电线 GLB 臂；通电时额外显示凹槽白条
/// `shorten_for_panel`：该面贴灯板时沿法线缩到 0.8，避免穿出面板
pub fn spawn_factory_wire_arm(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    face_index: usize,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
    powered: bool,
    shorten_for_panel: bool,
) {
    let Some(FactoryVisual::Wire { faces, power }) =
        assets.factory_visual(crate::game::blocks::BlockKind::Wire)
    else {
        return;
    };
    let transform = if shorten_for_panel {
        let axis = [
            IVec3::X,
            IVec3::NEG_X,
            IVec3::Y,
            IVec3::NEG_Y,
            IVec3::Z,
            IVec3::NEG_Z,
        ][face_index];
        Transform::from_scale(Vec3::new(
            if axis.x != 0 { 0.8 } else { 1.0 },
            if axis.y != 0 { 0.8 } else { 1.0 },
            if axis.z != 0 { 0.8 } else { 1.0 },
        ))
    } else {
        Transform::default()
    };
    if let Some(parts) = faces.get(face_index) {
        for part in parts {
            let material = if preview {
                part.preview_material.clone()
            } else {
                part.material.clone()
            };
            let mut child = parent.spawn((
                Mesh3d(part.mesh.clone()),
                MeshMaterial3d(material),
                transform,
            ));
            if let Some(icon_layer) = icon_layer {
                child.insert((icon_layer.clone(), BlockIconRenderEntity));
            }
        }
    }
    if powered {
        if let Some(parts) = power.get(face_index) {
            for part in parts {
                let mut child = parent.spawn((
                    Mesh3d(part.mesh.clone()),
                    MeshMaterial3d(part.material.clone()),
                    transform,
                ));
                if let Some(icon_layer) = icon_layer {
                    child.insert((icon_layer.clone(), BlockIconRenderEntity));
                }
            }
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
        ROD_BASE_LENGTH, pusher_rod_center_z, pusher_rod_length,
    };

    for part in parts {
        let mut translation = model_vec3(part.translation);
        let mut scale = model_vec3(part.scale);

        // 必须用 from_extension：实体要到下一帧才进 animate，若用 to 会首帧直接画成终点
        if part.mesh == ModelMesh::PusherHead {
            if let Some(animation) = pusher_animation {
                translation += Vec3::NEG_Z * animation.from_extension;
            }
        } else if part.mesh == ModelMesh::RodZ {
            let extension = pusher_animation
                .map(|animation| animation.from_extension)
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
        animation.duration.is_some_and(|duration| duration > 0.0)
            && animation.from_extension != animation.to_extension
    }) {
        if part.mesh == ModelMesh::PusherHead {
            child.insert(AnimatedPusher::new(animation, model_vec3(part.translation)));
        } else if part.mesh == ModelMesh::RodZ {
            child.insert(AnimatedPusherRod::new(animation, model_vec3(part.scale)));
        }
    }
}

fn model_vec3(value: [f32; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}
