use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game::world::animation::{
    AnimatedPusher, LifterDiskGlow, PusherAnimation, ScrollingConveyorBelt, SpinningDrillHead,
};
use crate::game::world::render_assets::{FactoryPartHandles, FactoryVisual, WorldRenderAssets};
use crate::game::world::rendering::BlockIconRenderEntity;

/// 生成方块模型零件（工厂 GLB）
pub fn spawn_model_parts(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    kind: crate::game::blocks::BlockKind,
    block_id: crate::game::blocks::BlockId,
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    let Some(visual) = assets.factory_visual(kind) else {
        return;
    };
    match visual {
        FactoryVisual::Static {
            parts,
            local_rotation,
        } => {
            spawn_factory_static(
                parent,
                assets,
                kind,
                parts,
                *local_rotation,
                block_id,
                icon_layer,
                preview,
            );
        }
        FactoryVisual::Drill { body, head } => {
            spawn_factory_drill(parent, assets, body, head, icon_layer, preview);
        }
        FactoryVisual::Pusher { body, stage, head } => {
            spawn_factory_pusher(
                parent,
                assets,
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
}

/// 生成静态工厂 GLB 零件（可选额外局部旋转）
fn spawn_factory_static(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    kind: crate::game::blocks::BlockKind,
    parts: &[FactoryPartHandles],
    local_rotation: Quat,
    block_id: crate::game::blocks::BlockId,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    if local_rotation == Quat::IDENTITY {
        for part in parts {
            spawn_factory_part(
                parent,
                assets,
                kind,
                part,
                Transform::default(),
                Some(block_id),
                icon_layer,
                preview,
            );
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
            spawn_factory_part(
                parent,
                assets,
                kind,
                part,
                Transform::default(),
                Some(block_id),
                icon_layer,
                preview,
            );
        }
    });
}

/// 生成钻头 Body / Head；Head 挂持续旋转组件
fn spawn_factory_drill(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    body: &[FactoryPartHandles],
    head: &[FactoryPartHandles],
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    for part in body {
        spawn_factory_part(
            parent,
            assets,
            crate::game::blocks::BlockKind::Drill,
            part,
            Transform::default(),
            None,
            icon_layer,
            preview,
        );
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
            spawn_factory_part(
                parent,
                assets,
                crate::game::blocks::BlockKind::Drill,
                part,
                Transform::default(),
                None,
                icon_layer,
                preview,
            );
        }
    });
}

/// 生成活塞 Body / Stage / Head
fn spawn_factory_pusher(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    body: &[FactoryPartHandles],
    stage: &[FactoryPartHandles],
    head: &[FactoryPartHandles],
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    for part in body {
        spawn_factory_part(
            parent,
            assets,
            crate::game::blocks::BlockKind::Pusher,
            part,
            Transform::default(),
            None,
            icon_layer,
            preview,
        );
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
            spawn_factory_part(
                parent,
                assets,
                crate::game::blocks::BlockKind::Pusher,
                part,
                Transform::default(),
                None,
                icon_layer,
                preview,
            );
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
            spawn_factory_part(
                parent,
                assets,
                crate::game::blocks::BlockKind::Pusher,
                part,
                Transform::default(),
                None,
                icon_layer,
                preview,
            );
        }
    });
}

/// 生成单段工厂 GLB 网格
fn spawn_factory_part(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    kind: crate::game::blocks::BlockKind,
    part: &FactoryPartHandles,
    transform: Transform,
    block_id: Option<crate::game::blocks::BlockId>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    let material = if preview {
        part.preview_material.clone()
    } else {
        part.material.clone()
    };
    let mut child = parent.spawn((
        Mesh3d(part.mesh.clone()),
        MeshMaterial3d(material.clone()),
        transform,
    ));
    if let Some(icon_layer) = icon_layer {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    // 传送带皮带：模拟时滚动 UV（颜色+法线共用 uv_transform）
    if !preview && icon_layer.is_none() && part.group.as_deref() == Some("Part_Belt") {
        child.insert(ScrollingConveyorBelt);
    }
    // 仅抬升器顶盘：模拟期自发光，通电关闭（旋转器也有 Part_Disk，不能共用）
    if !preview
        && icon_layer.is_none()
        && kind == crate::game::blocks::BlockKind::Lifter
        && part.group.as_deref() == Some("Part_Disk")
    {
        if let Some(block_id) = block_id.filter(|id| !id.is_none()) {
            child.insert(LifterDiskGlow {
                block_id,
                idle: material,
                lit: assets.lifter_disk_lit_material.clone(),
            });
        }
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

