//! 世界网格：Bevy Resource 包装 `oif_sim::WorldBlocks`

pub use oif_sim::world::grid::{
    grid_to_world, raycast_blocks, raycast_infinite_plane, world_to_grid,
    BlockSettings, ConverterMode, ConverterSettings, EditSelectionMode,
    GeneratorMode, GeneratorSettings, GoalSettings, MaterialFace, MaterialWeld, REACH,
    RollerSettings, SignDisplay, SignSettings, StamperSettings, StoredAcceptorStructure,
    TargetHit, TeleportSettings,
};

use bevy::prelude::*;
use oif_sim::world::grid::EditSelectionMode as SimEditSelectionMode;

use crate::shared::config::ConfigSelectionMode;

/// 游戏世界方块网格（Bevy Resource，Deref 到 oif-sim）
#[derive(Resource, Deref, DerefMut, Clone, Default)]
pub struct WorldBlocks(pub oif_sim::WorldBlocks);

/// 配置层框选 → 模拟侧框选，再调用 oif-sim 射线
pub fn raycast_edit_drag_grid(
    origin: Vec3,
    dir: Vec3,
    start: IVec3,
    mode: ConfigSelectionMode,
    camera_dir: Vec3,
    plane_normal: IVec3,
) -> Option<IVec3> {
    let mode = match mode {
        ConfigSelectionMode::Point => SimEditSelectionMode::Point,
        ConfigSelectionMode::Line => SimEditSelectionMode::Line,
        ConfigSelectionMode::Plane => SimEditSelectionMode::Plane,
    };
    oif_sim::world::grid::raycast_edit_drag_grid(
        origin,
        dir,
        start,
        mode,
        camera_dir,
        plane_normal,
    )
}
