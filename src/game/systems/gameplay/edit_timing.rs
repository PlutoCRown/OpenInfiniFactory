//! 编辑批处理耗时：上一批放置 / 删除（供斜杠调试面板）

use bevy::prelude::Resource;

/// 上一批放置/删除的墙钟耗时
#[derive(Resource, Default)]
pub struct EditBatchTiming {
    pub last_place_ms: Option<f64>,
    pub last_delete_ms: Option<f64>,
}
