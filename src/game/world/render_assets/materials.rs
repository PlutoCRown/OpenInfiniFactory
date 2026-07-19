//! 方块 / 模型 / 预览材质构造

use bevy::prelude::*;

use crate::game::blocks::{BlockKind, BlockPresent};

/// 纯色方块材质
pub(super) fn block_material(kind: BlockKind) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: kind.material(),
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    };
    if kind.is_transparent() {
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = kind.is_generated_marker();
    }
    material
}

/// 带贴图的方块材质
pub(super) fn textured_block_material(kind: BlockKind, texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material(),
        base_color_texture: Some(texture),
        perceptual_roughness: 0.94,
        reflectance: 0.10,
        ..default()
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

/// 工厂零件预览：幽灵色但不透明，避免多零件闪烁
pub(super) fn preview_model_material(material: StandardMaterial) -> StandardMaterial {
    // 不用 Blend：多零件 GLB 每帧重建时透明排序会闪；Opaque 幽灵色更稳
    let c = material.base_color.to_srgba();
    StandardMaterial {
        base_color: Color::srgba(
            c.red * 0.55 + 0.28,
            c.green * 0.55 + 0.30,
            c.blue * 0.55 + 0.34,
            1.0,
        ),
        base_color_texture: material.base_color_texture,
        normal_map_texture: material.normal_map_texture,
        emissive: material.emissive * 0.25,
        metallic: material.metallic * 0.35,
        perceptual_roughness: material.perceptual_roughness.max(0.75),
        reflectance: material.reflectance,
        alpha_mode: AlphaMode::Opaque,
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

/// 程序化零件用 sRGB 材质
pub(super) fn srgb_material(r: f32, g: f32, b: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        perceptual_roughness: 0.82,
        reflectance: 0.16,
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

/// 自发光模型零件材质
pub(super) fn emissive_material(
    r: f32,
    g: f32,
    b: f32,
    er: f32,
    eg: f32,
    eb: f32,
) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        emissive: Color::srgb(er, eg, eb).into(),
        perceptual_roughness: 0.72,
        reflectance: 0.10,
        ..default()
    }
}
