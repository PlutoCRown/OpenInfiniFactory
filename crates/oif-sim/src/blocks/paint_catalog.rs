//! 滚刷漆材料运行时目录：字符串 id ↔ PaintMaterialId

use std::sync::{LazyLock, Once, RwLock};

use serde::{Deserialize, Serialize};

use super::catalog_store::{
    clone_global, install_global, leak_str, with_global, with_global_mut, CatalogEntry, HasStringId,
    StringIdCatalog,
};

/// 滚刷漆材料句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PaintMaterialId(pub u16);

impl From<u16> for PaintMaterialId {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<PaintMaterialId> for u16 {
    fn from(id: PaintMaterialId) -> Self {
        id.0
    }
}

/// 单种滚刷漆材料的模拟侧定义
#[derive(Clone, Debug)]
pub struct PaintMaterialDef {
    pub string_id: String,
    /// i18n key，形如 `paint.red`（泄漏为 `'static` 供旧 API）
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
}

impl HasStringId for PaintMaterialDef {
    fn string_id(&self) -> &str {
        &self.string_id
    }
}

impl CatalogEntry for PaintMaterialDef {
    const LABEL: &'static str = "paint material";
}

/// 滚刷漆材料目录
pub type PaintMaterialCatalog = StringIdCatalog<PaintMaterialId, PaintMaterialDef>;

static CATALOG: LazyLock<RwLock<PaintMaterialCatalog>> =
    LazyLock::new(|| RwLock::new(PaintMaterialCatalog::new()));

static FALLBACK_ONCE: Once = Once::new();

/// 安装/替换全局滚刷漆目录（游戏加载资源包时调用）
pub fn install_paint_catalog(catalog: PaintMaterialCatalog) {
    install_global(&CATALOG, catalog);
}

/// 读取当前目录快照
pub fn paint_catalog() -> PaintMaterialCatalog {
    ensure_fallback_paint_catalog();
    clone_global(&CATALOG)
}

/// 按 string id 查句柄（读锁，不克隆整表）
pub fn paint_id_by_string(string_id: &str) -> Option<PaintMaterialId> {
    ensure_fallback_paint_catalog();
    with_global(&CATALOG, |c| c.id_by_string(string_id))
}

/// 按 id 查定义（无则 panic）
pub fn paint_def(id: PaintMaterialId) -> PaintMaterialDef {
    ensure_fallback_paint_catalog();
    with_global(&CATALOG, |c| {
        c.get(id)
            .cloned()
            .unwrap_or_else(|| panic!("unknown PaintMaterialId {}", id.0))
    })
}

/// 无资源包时的兜底（单测 / wasm）：注册红绿蓝黄，编号不保证跨会话稳定
pub fn ensure_fallback_paint_catalog() {
    FALLBACK_ONCE.call_once(|| {
        with_global_mut(&CATALOG, |catalog| {
            if !catalog.is_empty() {
                return;
            }
            for string_id in ["red", "green", "blue", "yellow"] {
                let name_key = leak_str(&format!("paint.{string_id}"));
                let short_name_key = leak_str(&format!("short.paint.{string_id}"));
                let description_key = leak_str(&format!("desc.paint.{string_id}"));
                catalog
                    .register(PaintMaterialDef {
                        string_id: string_id.to_string(),
                        name_key,
                        short_name_key,
                        description_key,
                    })
                    .expect("fallback paint id unique");
            }
        });
    });
}
