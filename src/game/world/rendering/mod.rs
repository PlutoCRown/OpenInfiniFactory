//! 世界渲染：场景实体、方块生成、预览与特效

mod block_icons;
mod components;
mod connectors;
pub(crate) mod depth_bias;
mod frame_ready;
mod fx;
pub(crate) mod goal_ghost;
mod goal_visual_sync;
mod icons;
mod lighting;
pub(crate) mod portal_material;
mod previews;
mod procedural_sky;
mod scene;
mod scene_chunks;
mod scene_mesh;
pub(crate) mod shadow_proxy;
pub(crate) mod shadow_proxy_material;
mod skybox;
pub(crate) mod spawn;
mod world_rebuild;

pub use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};

pub use block_icons::{
    LIGHT_PANEL_ICON_RELPATH, SELECTION_ICON_RELPATH, bakeable_block_icon_kinds,
    baked_block_icon_only_id, baked_block_icon_path, baked_block_icon_relpath,
    light_panel_icon_path, selection_icon_path,
};

pub(crate) use components::BlockIconRenderEntity;
pub(crate) use components::GeneratorConfigMaterialPreview;
pub use components::{
    AimFaceHighlight, BlockEntity, BlockEntityLayer, BlockIconAssets, BlockIconRenderRoot,
    DeleteBoundsOverlay, DeleteBoundsPart, EditPreview, FactoryDebugOverlay, GameplayScene,
    HoverMarker, HoverStructureBounds, PendingGeneratedPreview, PlacementPreview,
    SelectionBoundsOverlay, SelectionBoundsPart, StructureBounds, block_face_highlight_transform,
};
pub(crate) use connectors::{light_panel_transform, signal_neighbor_offsets};
pub use frame_ready::SceneFrameReady;
pub use fx::{
    spawn_acceptance_sparks, spawn_break_debris, spawn_laser_beams, spawn_weld_bursts,
    spawn_weld_sparks,
};
pub use goal_ghost::{GoalGhostMaterial, GoalGhostPlugin};
pub use icons::setup_block_icons;
pub use lighting::{environment_map_intensity, environment_map_light};
pub use portal_material::{PortalFlashQueue, PortalMaterial, PortalMaterialPlugin};
pub use previews::{
    despawn_edit_previews, spawn_block_preview, spawn_edit_preview, update_delete_bounds_overlay,
    update_selection_bounds_overlay,
};
pub use scene::{
    create_world_render_assets, gameplay_ssao, setup_scene, sync_ssao_settings,
    sync_vsync_settings, sync_window_mode_settings, teardown_playing_scene,
};
pub use scene_chunks::{
    SceneChunkMeshes, clear_scene_chunks, forget_scene_chunks, rebuild_all_scene_chunks,
    sync_scene_chunks_for_positions,
};
pub use shadow_proxy_material::ShadowProxyMaterial;
pub use skybox::SkyboxPlugin;
pub use spawn::{
    SpawnBlockOpts, SpawnMode, spawn_block, spawn_block_with_animation,
    spawn_block_with_timed_animation, spawn_pending_generated_block,
};
pub(crate) use spawn::{
    factory_debug_overlay_material, spawn_factory_debug_overlay, spawn_world_block_entity,
};
pub use world_rebuild::{
    despawn_world, rebuild_world, rebuild_world_with_animations,
    rebuild_world_with_runtime_animations, rebuild_world_with_timed_animations,
};

use crate::game::schedule::GameSet;
use crate::game::{material_blocks, scene_blocks};
use bevy::prelude::*;

/// 拥有资源包、世界渲染索引和相机呈现系统。
pub struct WorldRenderingPlugin;

impl Plugin for WorldRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::scene::BlockEntityIndex>()
            .init_resource::<SceneChunkMeshes>()
            .init_resource::<crate::game::cameras::GameplayRenderThrottle>()
            .init_resource::<scene_blocks::SceneBlockRegistry>()
            .init_resource::<material_blocks::MaterialBlockRegistry>()
            .init_resource::<material_blocks::StampMaterialRegistry>()
            .init_resource::<material_blocks::PaintMaterialRegistry>()
            .insert_resource(ClearColor(crate::game::cameras::MENU_CLEAR))
            .insert_resource(bevy::light::GlobalAmbientLight {
                color: Color::srgb(0.90, 0.94, 1.0),
                brightness: 340.0,
                affects_lightmapped_meshes: true,
            })
            .insert_resource(bevy::light::DirectionalLightShadowMap { size: 2048 })
            .add_plugins((
                frame_ready::SceneReadinessPlugin,
                SkyboxPlugin,
                GoalGhostPlugin,
                PortalMaterialPlugin,
                bevy::pbr::MaterialPlugin::<ShadowProxyMaterial>::default(),
            ))
            .add_systems(
                Startup,
                load_scene_blocks_on_startup.in_set(crate::game::schedule::GameStartupSet::Content),
            )
            .add_systems(
                Update,
                (
                    crate::game::world::animation::animate_blocks,
                    crate::scene::apply_pending_teleport_snaps,
                    crate::game::world::animation::scroll_conveyor_belts,
                    crate::game::world::animation::update_lifter_disk_glow,
                )
                    .chain()
                    .in_set(GameSet::Animation),
            )
            .add_systems(
                Update,
                (
                    sync_window_mode_settings,
                    sync_vsync_settings,
                    crate::game::cameras::sync_gameplay_view_image_size,
                    sync_ssao_settings,
                    lighting::sync_puzzle_lighting,
                    crate::game::cameras::sync_gameplay_render_rate,
                )
                    .chain()
                    .in_set(GameSet::View),
            );
    }
}

/// 启动时加载全局场景方块与材料/印花/滚刷资源包
pub(crate) fn load_scene_blocks_on_startup(
    mut scene_registry: ResMut<scene_blocks::SceneBlockRegistry>,
    mut material_registry: ResMut<material_blocks::MaterialBlockRegistry>,
    mut stamp_registry: ResMut<material_blocks::StampMaterialRegistry>,
    mut paint_registry: ResMut<material_blocks::PaintMaterialRegistry>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        // wasm 无同步列目录 API（HTTP AssetReader），打包资源扫盘不可用；与修 Android 前同类问题
        oif_sim::blocks::ensure_fallback_scene_catalog();
        oif_sim::blocks::ensure_fallback_material_catalog();
        oif_sim::blocks::ensure_fallback_stamp_catalog();
        oif_sim::blocks::ensure_fallback_paint_catalog();
        let _ = (
            &mut scene_registry,
            &mut material_registry,
            &mut stamp_registry,
            &mut paint_registry,
        );
        bevy::log::info!(
            "scene/material packs: wasm has no sync packaged-asset scan; using fallback catalogs"
        );
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        match scene_blocks::load_global_scene_blocks(&mut scene_registry) {
            Ok(()) => bevy::log::info!(
                "loaded {} scene block packs",
                scene_registry.ordered_kinds().len()
            ),
            Err(err) => {
                bevy::log::error!("failed to load scene blocks: {err}; using fallback catalog");
                oif_sim::blocks::ensure_fallback_scene_catalog();
            }
        }
        match material_blocks::load_global_material_packs(material_blocks::MaterialPackRegistries {
            materials: &mut material_registry,
            stamps: &mut stamp_registry,
            paints: &mut paint_registry,
        }) {
            Ok(()) => bevy::log::info!(
                "loaded {} material / {} stamp / {} paint packs",
                material_registry.ordered().count(),
                stamp_registry.ordered().count(),
                paint_registry.ordered().count()
            ),
            Err(err) => {
                bevy::log::error!("failed to load material packs: {err}; using fallback catalogs");
                oif_sim::blocks::ensure_fallback_material_catalog();
                oif_sim::blocks::ensure_fallback_stamp_catalog();
                oif_sim::blocks::ensure_fallback_paint_catalog();
            }
        }
    }
}
