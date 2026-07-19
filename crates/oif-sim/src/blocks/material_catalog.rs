//! 材料方块运行时目录：字符串 id ↔ MaterialBlockId，以及 connectable/fragile 等模拟元数据

use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use super::catalog_store::{
    clone_global, install_global, leak_str, with_global, with_global_mut, CatalogEntry, HasStringId,
    StringIdCatalog,
};
use super::{rgb, ColorSpec};

/// 找不到资源包时使用的兜底材料 string id
pub const FALLBACK_MATERIAL_STRING_ID: &str = "fallback";

/// 材料方块句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct MaterialBlockId(pub u16);

impl From<u16> for MaterialBlockId {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<MaterialBlockId> for u16 {
    fn from(id: MaterialBlockId) -> Self {
        id.0
    }
}

/// 单种材料方块的模拟侧定义
#[derive(Clone, Debug)]
pub struct MaterialBlockDef {
    pub string_id: String,
    /// i18n key，形如 `block.basic`（泄漏为 `'static` 供旧 API）
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
    pub connectable: [bool; 6],
    pub fragile: bool,
    /// 是否可朝向（放置时 R 旋转，存档持久化 facing）
    pub directional: bool,
    pub color: ColorSpec,
}

impl HasStringId for MaterialBlockDef {
    fn string_id(&self) -> &str {
        &self.string_id
    }
}

impl CatalogEntry for MaterialBlockDef {
    const LABEL: &'static str = "material block";
}

/// 材料方块目录
pub type MaterialBlockCatalog = StringIdCatalog<MaterialBlockId, MaterialBlockDef>;

impl MaterialBlockCatalog {
    /// 解析 string id；未知则返回兜底材料
    pub fn resolve_id(&self, string_id: &str) -> MaterialBlockId {
        self.id_by_string(string_id)
            .unwrap_or_else(|| self.fallback_id())
    }

    /// 目录内兜底材料（必存在）
    pub fn fallback_id(&self) -> MaterialBlockId {
        self.id_by_string(FALLBACK_MATERIAL_STRING_ID)
            .or_else(|| (!self.is_empty()).then_some(MaterialBlockId(0)))
            .expect("material catalog must contain fallback")
    }
}

static CATALOG: LazyLock<RwLock<MaterialBlockCatalog>> =
    LazyLock::new(|| RwLock::new(MaterialBlockCatalog::new()));

/// 确保目录里有兜底材料（外观由渲染层棋盘贴图负责）
fn ensure_fallback_entry(catalog: &mut MaterialBlockCatalog) {
    if catalog.id_by_string(FALLBACK_MATERIAL_STRING_ID).is_some() {
        return;
    }
    catalog
        .register(MaterialBlockDef {
            string_id: FALLBACK_MATERIAL_STRING_ID.to_string(),
            name_key: leak_str("block.fallback"),
            short_name_key: leak_str("short.fallback"),
            description_key: leak_str("desc.fallback"),
            connectable: [true; 6],
            fragile: false,
            directional: false,
            // UI 无贴图时的色块提示（洋红）
            color: rgb(1.0, 0.0, 1.0),
        })
        .expect("fallback material id unique");
}

/// 安装/替换全局材料目录；保证兜底材料存在
pub fn install_material_catalog(mut catalog: MaterialBlockCatalog) {
    ensure_fallback_entry(&mut catalog);
    install_global(&CATALOG, catalog);
}

/// 读取当前目录快照
pub fn material_catalog() -> MaterialBlockCatalog {
    ensure_fallback_material_catalog();
    clone_global(&CATALOG)
}

/// 当前兜底材料 id
pub fn fallback_material_id() -> MaterialBlockId {
    ensure_fallback_material_catalog();
    with_global(&CATALOG, |c| c.fallback_id())
}

/// 按 string id 解析；未知 → 兜底
pub fn resolve_material_id(string_id: &str) -> MaterialBlockId {
    ensure_fallback_material_catalog();
    with_global(&CATALOG, |c| c.resolve_id(string_id))
}

/// 按 id 查定义；未知 id → 兜底定义
pub fn material_def(id: MaterialBlockId) -> MaterialBlockDef {
    ensure_fallback_material_catalog();
    with_global(&CATALOG, |catalog| {
        catalog
            .get(id)
            .cloned()
            .unwrap_or_else(|| catalog.get(catalog.fallback_id()).cloned().expect("fallback def"))
    })
}

/// 确保兜底材料已注册
pub fn ensure_fallback_material_catalog() {
    if with_global(&CATALOG, |c| c.id_by_string(FALLBACK_MATERIAL_STRING_ID).is_some()) {
        return;
    }
    with_global_mut(&CATALOG, ensure_fallback_entry);
}
