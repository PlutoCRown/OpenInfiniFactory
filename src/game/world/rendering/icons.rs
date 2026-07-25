//! 方块 UI 图标：全部读预烘焙 PNG（场景/材料/印花/工厂/系统块）

use bevy::prelude::*;

use super::block_icons::{bakeable_block_icon_kinds, baked_block_icon_path, selection_icon_path};
use super::components::{
    BlockIconAssets, BlockIconRenderCamera, BlockIconRenderRoot, BlockIconRenderState,
};
use crate::game::blocks::BlockKind;
use crate::game::material_blocks::{
    MaterialBlockRegistry, PaintMaterialRegistry, StampMaterialRegistry,
};
use crate::game::scene_blocks::{SceneBlockRegistry, load_icon_png};

/// 为 UI 准备方块图标：一律加载预烘焙 PNG，不再离屏渲染
pub fn setup_block_icons(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    scene_registry: Res<SceneBlockRegistry>,
    material_registry: Res<MaterialBlockRegistry>,
    stamp_registry: Res<StampMaterialRegistry>,
    paint_registry: Res<PaintMaterialRegistry>,
) {
    let mut icon_assets = BlockIconAssets::default();

    for kind in scene_registry.ordered_kinds() {
        let Some(presentation) = scene_registry.get_kind(kind) else {
            continue;
        };
        let Some(icon_path) = presentation.icon_path.as_ref() else {
            bevy::log::warn!(
                "scene block `{}` missing icon.png (run bake_scene_icons)",
                presentation.string_id
            );
            continue;
        };
        match load_icon_png(icon_path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!("failed to load icon {}", icon_path.display());
            }
        }
    }

    for presentation in material_registry.ordered() {
        let kind = BlockKind::Material(presentation.id);
        let Some(icon_path) = presentation.icon_path.as_ref() else {
            bevy::log::warn!("material `{}` missing icon.png", presentation.string_id);
            continue;
        };
        match load_icon_png(icon_path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!("failed to load icon {}", icon_path.display());
            }
        }
    }
    for presentation in stamp_registry.ordered() {
        let kind = BlockKind::Stamp(presentation.id);
        let Some(icon_path) = presentation.icon_path.as_ref() else {
            bevy::log::warn!("stamp `{}` missing icon.png", presentation.string_id);
            continue;
        };
        match load_icon_png(icon_path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!("failed to load icon {}", icon_path.display());
            }
        }
    }

    for presentation in paint_registry.ordered() {
        match load_icon_png(&presentation.texture_path, &mut images) {
            Some(handle) => {
                icon_assets.paints.insert(presentation.id, handle);
            }
            None => {
                bevy::log::warn!(
                    "failed to load paint texture {}",
                    presentation.texture_path.display()
                );
            }
        }
    }

    {
        use crate::game::blocks::{fallback_material_id, fallback_scene_id};
        let icon = images.add(crate::game::world::procedural_textures::missing_texture_image());
        icon_assets
            .icons
            .insert(BlockKind::Material(fallback_material_id()), icon.clone());
        icon_assets
            .icons
            .insert(BlockKind::Scene(fallback_scene_id()), icon);
    }

    for kind in bakeable_block_icon_kinds() {
        let Some(path) = baked_block_icon_path(kind) else {
            continue;
        };
        match load_icon_png(&path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!(
                    "factory/system icon missing {} (run bake_scene_icons --factory-only)",
                    path.display()
                );
            }
        }
    }

    let selection_path = selection_icon_path();
    match load_icon_png(&selection_path, &mut images) {
        Some(handle) => {
            icon_assets.selection = Some(handle);
        }
        None => {
            bevy::log::warn!(
                "selection icon missing {} (run bake_scene_icons --factory-only)",
                selection_path.display()
            );
        }
    }

    commands.insert_resource(icon_assets);
}

/// 兼容旧离屏拍帧清理（现已无 BlockIconRenderState，通常为空操作）
pub fn retire_block_icon_renderers(
    mut commands: Commands,
    state: Option<ResMut<BlockIconRenderState>>,
    render_entities: Query<Entity, With<BlockIconRenderRoot>>,
    mut cameras: Query<&mut Camera, With<BlockIconRenderCamera>>,
) {
    let Some(mut state) = state else {
        return;
    };
    if state.frames_remaining > 0 {
        state.frames_remaining -= 1;
        return;
    }

    for mut camera in &mut cameras {
        camera.is_active = false;
    }
    for entity in &render_entities {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BlockIconRenderState>();
}
