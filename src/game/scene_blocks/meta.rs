//! 场景方块 meta.json 解析

use serde::Deserialize;

/// 资源包元数据（对应 schemas/scene_block.meta.schema.json）
#[derive(Debug, Deserialize)]
pub struct SceneBlockMetaFile {
    #[serde(rename = "$schema")]
    pub _schema: Option<String>,
    pub id: String,
    #[serde(default = "default_collision")]
    pub collision: bool,
    #[serde(default = "default_connectable")]
    pub connectable: [bool; 6],
}

fn default_collision() -> bool {
    true
}

fn default_connectable() -> [bool; 6] {
    [true; 6]
}
