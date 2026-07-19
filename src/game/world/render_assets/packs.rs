//! 场景 / 材料 / 印花的 model.glb 与 texture 装载

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;

use crate::game::blocks::BlockKind;

use super::materials::{preview_block_material, preview_model_material, scene_color_material};

/// 把 model.glb 或 texture.png（可选 normal.png）装进 scene_* / block_materials
/// 有 model.glb 时只走 GLB；无模型的纯立方体才用 texture/normal.png
/// 印花薄板虽常为 24 顶点，也不写入 face_uvs，避免被当成整格 AO 立方体
pub(super) fn insert_configured_pack(
    kind: BlockKind,
    model_path: Option<&Path>,
    texture_path: Option<&Path>,
    normal_path: Option<&Path>,
    fallback_color: Color,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    scene_meshes: &mut HashMap<BlockKind, Handle<Mesh>>,
    scene_face_uvs: &mut HashMap<BlockKind, [[f32; 2]; 24]>,
    scene_block_materials: &mut HashMap<BlockKind, Handle<StandardMaterial>>,
    block_materials: &mut HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: &mut HashMap<BlockKind, Handle<StandardMaterial>>,
) {
    if let Some(model_path) = model_path {
        match crate::game::scene_blocks::load_scene_glb(model_path, meshes, materials, images) {
            Ok(loaded) => {
                if !matches!(kind, BlockKind::Stamp(_)) {
                    if let Some(uvs) = loaded.face_uvs {
                        scene_face_uvs.insert(kind, uvs);
                    }
                }
                scene_meshes.insert(kind, loaded.mesh);
                if let Some(base) = materials.get(&loaded.material).cloned() {
                    preview_materials.insert(kind, materials.add(preview_model_material(base)));
                }
                scene_block_materials.insert(kind, loaded.material.clone());
                block_materials.insert(kind, loaded.material);
                return;
            }
            Err(err) => {
                bevy::log::error!(
                    "configured pack glb load failed ({}): {err}",
                    kind.name_key()
                );
            }
        }
    }

    if let Some(texture_path) = texture_path {
        if let Some(texture) =
            crate::game::scene_blocks::load_block_texture_png(texture_path, images)
        {
            let normal = normal_path.and_then(|path| {
                let handle = crate::game::scene_blocks::load_block_normal_png(path, images);
                if handle.is_none() {
                    bevy::log::error!("configured pack normal load failed: {}", path.display());
                }
                handle
            });
            let mut material = StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture.clone()),
                normal_map_texture: normal.clone(),
                perceptual_roughness: 0.94,
                reflectance: 0.10,
                ..default()
            };
            if kind.is_transparent() {
                material.alpha_mode = AlphaMode::Blend;
            }
            let material = materials.add(material);
            preview_materials.insert(
                kind,
                materials.add(preview_block_material(kind, Some(texture), normal)),
            );
            scene_block_materials.insert(kind, material.clone());
            block_materials.insert(kind, material);
            return;
        }
        bevy::log::error!(
            "configured pack texture load failed: {}",
            texture_path.display()
        );
    }

    let fallback = materials.add(scene_color_material(fallback_color));
    scene_block_materials.insert(kind, fallback.clone());
    block_materials.insert(kind, fallback);
}
