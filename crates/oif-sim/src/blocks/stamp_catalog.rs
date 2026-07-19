//! 印花材料运行时目录：字符串 id ↔ StampMaterialId，以及 fragile/color 等模拟元数据

use std::sync::{LazyLock, Once, RwLock};

use serde::{Deserialize, Serialize};

use super::catalog_store::{
    clone_global, install_global, leak_str, with_global, with_global_mut, CatalogEntry, HasStringId,
    StringIdCatalog,
};
use super::{rgb, ColorSpec};

/// 印花材料句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StampMaterialId(pub u16);

impl From<u16> for StampMaterialId {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<StampMaterialId> for u16 {
    fn from(id: StampMaterialId) -> Self {
        id.0
    }
}

/// 单种印花材料的模拟侧定义
#[derive(Clone, Debug)]
pub struct StampMaterialDef {
    pub string_id: String,
    /// i18n key，形如 `stamp.red`（泄漏为 `'static` 供旧 API）
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
    pub fragile: bool,
    pub color: ColorSpec,
}

impl HasStringId for StampMaterialDef {
    fn string_id(&self) -> &str {
        &self.string_id
    }
}

impl CatalogEntry for StampMaterialDef {
    const LABEL: &'static str = "stamp material";
}

/// 印花材料目录
pub type StampMaterialCatalog = StringIdCatalog<StampMaterialId, StampMaterialDef>;

/// 兜底印花四色种子表（红绿蓝黄）
pub const FALLBACK_STAMP_SEED_COLORS: [(&str, ColorSpec); 4] = [
    ("red", rgb(242.0 / 255.0, 31.0 / 255.0, 26.0 / 255.0)),
    ("green", rgb(51.0 / 255.0, 209.0 / 255.0, 71.0 / 255.0)),
    ("blue", rgb(46.0 / 255.0, 107.0 / 255.0, 242.0 / 255.0)),
    ("yellow", rgb(255.0 / 255.0, 214.0 / 255.0, 46.0 / 255.0)),
];

/// 按 string id 查种子色；未知则灰
pub fn stamp_seed_color(string_id: &str) -> ColorSpec {
    FALLBACK_STAMP_SEED_COLORS
        .iter()
        .find(|(id, _)| *id == string_id)
        .map(|(_, c)| *c)
        .unwrap_or(rgb(0.7, 0.7, 0.7))
}

static CATALOG: LazyLock<RwLock<StampMaterialCatalog>> =
    LazyLock::new(|| RwLock::new(StampMaterialCatalog::new()));

static FALLBACK_ONCE: Once = Once::new();

/// 安装/替换全局印花目录（游戏加载资源包时调用）
pub fn install_stamp_catalog(catalog: StampMaterialCatalog) {
    install_global(&CATALOG, catalog);
}

/// 读取当前目录快照
pub fn stamp_catalog() -> StampMaterialCatalog {
    ensure_fallback_stamp_catalog();
    clone_global(&CATALOG)
}

/// 按 string id 查句柄（读锁，不克隆整表）
pub fn stamp_id_by_string(string_id: &str) -> Option<StampMaterialId> {
    ensure_fallback_stamp_catalog();
    with_global(&CATALOG, |c| c.id_by_string(string_id))
}

/// 按 id 查定义（无则 panic，用于已解析的 BlockKind::Stamp）
pub fn stamp_def(id: StampMaterialId) -> StampMaterialDef {
    ensure_fallback_stamp_catalog();
    with_global(&CATALOG, |c| {
        c.get(id)
            .cloned()
            .unwrap_or_else(|| panic!("unknown StampMaterialId {}", id.0))
    })
}

/// 无资源包时的兜底（单测 / wasm）：注册红绿蓝黄，编号不保证跨会话稳定
pub fn ensure_fallback_stamp_catalog() {
    FALLBACK_ONCE.call_once(|| {
        with_global_mut(&CATALOG, |catalog| {
            if !catalog.is_empty() {
                return;
            }
            for &(string_id, color) in &FALLBACK_STAMP_SEED_COLORS {
                let name_key = leak_str(&format!("stamp.{string_id}"));
                let short_name_key = leak_str(&format!("short.stamp.{string_id}"));
                let description_key = leak_str(&format!("desc.stamp.{string_id}"));
                catalog
                    .register(StampMaterialDef {
                        string_id: string_id.to_string(),
                        name_key,
                        short_name_key,
                        description_key,
                        fragile: false,
                        color,
                    })
                    .expect("fallback stamp id unique");
            }
        });
    });
}
