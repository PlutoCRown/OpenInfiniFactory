//! 方块 / 模型 / 预览材质构造

use bevy::prelude::*;

#[cfg(not(target_os = "android"))]
use crate::game::blocks::PersistentLayer;
use crate::game::blocks::{BlockKind, BlockPresent};
#[cfg(not(target_os = "android"))]
use crate::game::world::rendering::depth_bias;

/// 纯色方块材质
pub(super) fn block_material(kind: BlockKind) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: kind.material(),
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    };
    if kind.is_transparent() {
        material.unlit = kind.is_generated_marker();
        #[cfg(target_os = "android")]
        {
            // 部分 Android GPU 对带深度偏置的透明壳不稳定，移动端用不透明壳保证可见。
            material.base_color = kind.material().with_alpha(1.0);
            material.alpha_mode = AlphaMode::Opaque;
        }
        #[cfg(not(target_os = "android"))]
        {
            material.alpha_mode = AlphaMode::Blend;
        }
    }
    apply_system_shell_bias(&mut material, kind);
    material
}

/// 带贴图的方块材质
pub(super) fn textured_block_material(kind: BlockKind, texture: Handle<Image>) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: kind.material(),
        base_color_texture: Some(texture),
        perceptual_roughness: 0.94,
        reflectance: 0.10,
        ..default()
    };
    apply_system_shell_bias(&mut material, kind);
    material
}

/// 谜题系统满格壳：与邻接工厂/材料共面时用 bias 分层（替代旧 1.05 缩放）
fn apply_system_shell_bias(_material: &mut StandardMaterial, _kind: BlockKind) {
    #[cfg(not(target_os = "android"))]
    if _kind.is_system_block() && _kind.persistent_layer() == Some(PersistentLayer::Puzzle) {
        _material.depth_bias = depth_bias::SYSTEM_SHELL;
    }
}

/// 半透明放置预览材质
pub(super) fn preview_block_material(
    kind: BlockKind,
    texture: Option<Handle<Image>>,
    normal: Option<Handle<Image>>,
) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material().with_alpha(0.46),
        base_color_texture: texture,
        normal_map_texture: normal,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.94,
        reflectance: 0.08,
        ..default()
    }
}

/// 模型/场景预览：幽灵色；原本不透明的保持 Opaque（多零件不闪），Blend/Mask 保留镂空
pub(super) fn preview_model_material(material: StandardMaterial) -> StandardMaterial {
    let c = material.base_color.to_srgba();
    // 源材质已是透明/镂空时必须保留 alpha_mode，否则贴图透明区会按 RGB（常为黑）实心画出
    let keep_cutout = !matches!(material.alpha_mode, AlphaMode::Opaque);
    let alpha = if matches!(material.alpha_mode, AlphaMode::Blend) {
        c.alpha * 0.46
    } else {
        1.0
    };
    StandardMaterial {
        base_color: Color::srgba(
            c.red * 0.55 + 0.28,
            c.green * 0.55 + 0.30,
            c.blue * 0.55 + 0.34,
            alpha,
        ),
        base_color_texture: material.base_color_texture,
        normal_map_texture: material.normal_map_texture,
        metallic_roughness_texture: material.metallic_roughness_texture,
        occlusion_texture: material.occlusion_texture,
        emissive: material.emissive * 0.25,
        metallic: material.metallic * 0.35,
        perceptual_roughness: material.perceptual_roughness.max(0.75),
        reflectance: material.reflectance,
        alpha_mode: if keep_cutout {
            material.alpha_mode
        } else {
            AlphaMode::Opaque
        },
        cull_mode: material.cull_mode,
        unlit: false,
        ..default()
    }
}

/// 场景兜底纯色材质
pub(super) fn scene_color_material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        perceptual_roughness: 0.96,
        reflectance: 0.08,
        ..default()
    }
}

/// 带贴图的模型零件材质
pub(super) fn textured_model_material(
    base_color: Color,
    texture: Handle<Image>,
) -> StandardMaterial {
    StandardMaterial {
        base_color,
        base_color_texture: Some(texture),
        perceptual_roughness: 0.90,
        reflectance: 0.12,
        ..default()
    }
}
