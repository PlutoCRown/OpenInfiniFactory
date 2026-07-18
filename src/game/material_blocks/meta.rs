//! 材料 / 印花 / 滚刷资源包 meta.json 解析

use serde::Deserialize;

/// 材料方块元数据（对应 schemas/material_block.meta.schema.json）
#[derive(Debug, Deserialize)]
pub struct MaterialBlockMetaFile {
    #[serde(rename = "$schema")]
    pub _schema: Option<String>,
    pub id: String,
    #[serde(default)]
    pub fragile: bool,
    #[serde(default)]
    pub directional: bool,
    #[serde(default = "default_connectable")]
    pub connectable: [bool; 6],
}

/// 印花材料元数据（对应 schemas/stamp_material.meta.schema.json）
#[derive(Debug, Deserialize)]
pub struct StampMaterialMetaFile {
    #[serde(rename = "$schema")]
    pub _schema: Option<String>,
    pub id: String,
    #[serde(default)]
    pub fragile: bool,
}

/// 滚刷漆材料元数据（对应 schemas/paint_material.meta.schema.json）
#[derive(Debug, Deserialize)]
pub struct PaintMaterialMetaFile {
    #[serde(rename = "$schema")]
    pub _schema: Option<String>,
    pub id: String,
}

fn default_connectable() -> [bool; 6] {
    [true; 6]
}
