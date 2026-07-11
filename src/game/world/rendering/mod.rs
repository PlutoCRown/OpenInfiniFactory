//! 世界渲染：场景实体、方块生成、预览与特效

mod components;
mod connectors;
mod fx;
mod icons;
mod previews;
mod scene;
mod scene_mesh;
mod spawn;
mod world_rebuild;

pub use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};

pub use components::{
    block_face_highlight_transform, AimFaceHighlight, BlockEntity, BlockEntityLayer,
    BlockIconAssets, BlockIconRenderCamera, BlockIconRenderRoot, BlockIconRenderState, EditPreview,
    GameplayScene, HoverMarker, HoverStructureBounds, PendingGeneratedPreview, PlacementPreview,
    StructureBounds,
};
pub(crate) use components::BlockIconRenderEntity;
pub use fx::{spawn_acceptance_sparks, spawn_laser_beams, spawn_weld_sparks};
pub use icons::{retire_block_icon_renderers, setup_block_icons};
pub use previews::{
    despawn_edit_previews, despawn_pending_generated_previews, spawn_block_preview,
    spawn_delete_bounds_preview, spawn_edit_preview,
};
pub use scene::{
    setup_scene, sync_shadow_settings, sync_vsync_settings, teardown_playing_scene,
};
pub use spawn::{
    spawn_block, spawn_block_with_animation, spawn_block_with_timed_animation,
    spawn_pending_generated_block,
};
pub(crate) use spawn::spawn_world_block_entity;
pub(crate) use connectors::signal_neighbor_offsets;
pub use world_rebuild::{
    despawn_world, rebuild_world, rebuild_world_for_debug_state, rebuild_world_on_enter,
    rebuild_world_with_animations, rebuild_world_with_animations_for_debug_state,
    rebuild_world_with_runtime_animations, rebuild_world_with_runtime_animations_for_debug_state,
    rebuild_world_with_timed_animations,
};
