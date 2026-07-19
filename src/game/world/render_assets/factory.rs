//! 工厂 GLB 零件加载与拆分

use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::blocks::BlockKind;
use crate::game::scene_blocks::{FactoryGltfPart, load_factory_glb};

use super::materials::preview_model_material;

/// 工厂 GLB 单个可渲染零件（含编辑预览材质）
#[derive(Clone)]
pub struct FactoryPartHandles {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub preview_material: Handle<StandardMaterial>,
}

/// 工厂块从 assets/factory_blocks 加载的外观
#[derive(Clone)]
pub enum FactoryVisual {
    /// 静态整块零件
    Static {
        parts: Vec<FactoryPartHandles>,
        /// 相对方块朝向的额外旋转（DownWelder 俯仰、反向传送带翻底等）
        local_rotation: Quat,
    },
    /// 钻头：Body 静、Head 模拟时绕前进轴转
    Drill {
        body: Vec<FactoryPartHandles>,
        head: Vec<FactoryPartHandles>,
    },
    /// 活塞 / 拦截器：Body 静、Stage 半行程、Head 全行程
    Pusher {
        body: Vec<FactoryPartHandles>,
        stage: Vec<FactoryPartHandles>,
        head: Vec<FactoryPartHandles>,
    },
    /// 电线六向臂：索引对齐 signal_neighbor_offsets；power 为通电凹槽灯
    Wire {
        faces: [Vec<FactoryPartHandles>; 6],
        power: [Vec<FactoryPartHandles>; 6],
    },
}

/// 加载灯面板 mesh（几何已烘焙；通电材质由引擎切换）
pub(super) fn load_light_panel_mesh(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Handle<Mesh> {
    use std::path::PathBuf;

    let path = PathBuf::from(crate::shared::platform::asset_path())
        .join("factory_blocks")
        .join("light_panel")
        .join("model.glb");
    match load_factory_glb(&path, meshes, materials, images) {
        Ok(mut parts) => {
            if let Some(part) = parts.pop() {
                return part.mesh;
            }
            bevy::log::warn!("light_panel glb has no mesh primitives");
        }
        Err(err) => {
            bevy::log::warn!("factory glb load failed (light_panel): {err}");
        }
    }
    meshes.add(Cuboid::new(1.0, 0.1, 1.0))
}

/// 扫描 factory_blocks 目录，按种类装成 FactoryVisual
pub(super) fn load_factory_visuals(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> HashMap<BlockKind, FactoryVisual> {
    use std::f32::consts::{FRAC_PI_2, PI};
    use std::path::PathBuf;

    let root = PathBuf::from(crate::shared::platform::asset_path()).join("factory_blocks");
    let mut map = HashMap::new();

    // 反向传送带：绕局部 X 翻 180°，顶面到底面
    let static_dirs: &[(BlockKind, &str, Quat)] = &[
        (BlockKind::Platform, "platform", Quat::IDENTITY),
        (BlockKind::Conveyor, "conveyor", Quat::IDENTITY),
        (
            BlockKind::ReverseConveyor,
            "conveyor",
            Quat::from_rotation_x(PI),
        ),
        (BlockKind::Rotator, "rotator", Quat::IDENTITY),
        (BlockKind::CounterRotator, "counter_rotator", Quat::IDENTITY),
        (BlockKind::Detector, "detector", Quat::IDENTITY),
        (BlockKind::DownDetector, "detector", Quat::IDENTITY),
        (BlockKind::Lifter, "lifter", Quat::IDENTITY),
        (BlockKind::Welder, "welder", Quat::IDENTITY),
        (
            BlockKind::DownWelder,
            "welder",
            Quat::from_rotation_x(-FRAC_PI_2),
        ),
        (BlockKind::Laser, "laser", Quat::IDENTITY),
        (BlockKind::Mirror, "mirror", Quat::IDENTITY),
        (BlockKind::VerticalMirror, "vertical_mirror", Quat::IDENTITY),
        (BlockKind::Splitter, "splitter", Quat::IDENTITY),
        (BlockKind::SuctionCup, "suction_cup", Quat::IDENTITY),
    ];
    for &(kind, dir, local_rotation) in static_dirs {
        let path = root.join(dir).join("model.glb");
        match load_factory_glb(&path, meshes, materials, images) {
            Ok(raw) => {
                map.insert(
                    kind,
                    FactoryVisual::Static {
                        parts: factory_parts_with_preview(raw, materials),
                        local_rotation,
                    },
                );
            }
            Err(err) => {
                bevy::log::warn!("factory glb load failed ({}): {err}", kind.name_key());
            }
        }
    }

    let drill_path = root.join("drill").join("model.glb");
    match load_factory_glb(&drill_path, meshes, materials, images) {
        Ok(raw) => match split_drill_parts(raw, materials) {
            Some(visual) => {
                map.insert(BlockKind::Drill, visual);
            }
            None => {
                bevy::log::warn!("factory drill glb missing Body/Head");
            }
        },
        Err(err) => {
            bevy::log::warn!("factory glb load failed (drill): {err}");
        }
    }

    for &(kind, dir) in &[
        (BlockKind::Pusher, "pusher"),
        (BlockKind::Blocker, "blocker"),
    ] {
        let path = root.join(dir).join("model.glb");
        match load_factory_glb(&path, meshes, materials, images) {
            Ok(raw) => match split_pusher_parts(raw, materials) {
                Some(visual) => {
                    map.insert(kind, visual);
                }
                None => {
                    bevy::log::warn!(
                        "factory pusher glb missing Body/Stage/Head ({})",
                        kind.name_key()
                    );
                }
            },
            Err(err) => {
                bevy::log::warn!("factory glb load failed ({}): {err}", kind.name_key());
            }
        }
    }

    let wire_path = root.join("wire").join("model.glb");
    match load_factory_glb(&wire_path, meshes, materials, images) {
        Ok(raw) => match split_wire_faces(raw, materials) {
            Some(visual) => {
                map.insert(BlockKind::Wire, visual);
            }
            None => {
                bevy::log::warn!("factory wire glb missing Pos/Neg face groups");
            }
        },
        Err(err) => {
            bevy::log::warn!("factory glb load failed (wire): {err}");
        }
    }

    map
}

/// 给工厂零件补半透明预览材质
fn factory_parts_with_preview(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<FactoryPartHandles> {
    raw.into_iter()
        .map(|part| {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        })
        .collect()
}

/// 按 Body / Head 拆钻头零件
fn split_drill_parts(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<FactoryVisual> {
    let mut body = Vec::new();
    let mut head = Vec::new();
    for part in raw {
        let group = part.group.as_deref().unwrap_or("");
        let handles = {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        };
        match group {
            "Head" => head.push(handles),
            _ => body.push(handles),
        }
    }
    if body.is_empty() || head.is_empty() {
        return None;
    }
    Some(FactoryVisual::Drill { body, head })
}

/// 按 Body / Stage / Head 拆活塞零件
fn split_pusher_parts(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<FactoryVisual> {
    let mut body = Vec::new();
    let mut stage = Vec::new();
    let mut head = Vec::new();
    for part in raw {
        let group = part.group.as_deref().unwrap_or("");
        let handles = {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        };
        match group {
            "Body" => body.push(handles),
            "Stage" => stage.push(handles),
            "Head" => head.push(handles),
            _ => body.push(handles),
        }
    }
    if body.is_empty() || head.is_empty() {
        return None;
    }
    Some(FactoryVisual::Pusher { body, stage, head })
}

/// 按 PosX…NegZ / PosX_Power… 拆电线六向臂与通电条
fn split_wire_faces(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<FactoryVisual> {
    let mut faces: [Vec<FactoryPartHandles>; 6] = Default::default();
    let mut power: [Vec<FactoryPartHandles>; 6] = Default::default();
    let mut any = false;
    for part in raw {
        let group = part.group.as_deref().unwrap_or("");
        let (is_power, face_name) = match group.strip_suffix("_Power") {
            Some(face) => (true, face),
            None => (false, group),
        };
        let Some(index) = wire_face_index(Some(face_name)) else {
            continue;
        };
        any = true;
        let mut handles = {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        };
        // 通电条：白自发光（须 lit，否则 emissive 不进 HDR，Bloom 无效）
        if is_power {
            handles.material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(16.0, 16.0, 16.0, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            });
            handles.preview_material = handles.material.clone();
            power[index].push(handles);
        } else {
            faces[index].push(handles);
        }
    }
    if !any {
        return None;
    }
    Some(FactoryVisual::Wire { faces, power })
}

/// 电线节点名 → signal_neighbor_offsets 下标
fn wire_face_index(group: Option<&str>) -> Option<usize> {
    match group? {
        "PosX" => Some(0),
        "NegX" => Some(1),
        "PosY" => Some(2),
        "NegY" => Some(3),
        "PosZ" => Some(4),
        "NegZ" => Some(5),
        _ => None,
    }
}
