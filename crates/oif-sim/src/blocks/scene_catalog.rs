//! 场景方块运行时目录：字符串 id ↔ SceneBlockId，以及碰撞/connectable 等模拟元数据

use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use super::catalog_store::{
    clone_global, install_global, with_global, with_global_mut, CatalogEntry, HasStringId,
    StringIdCatalog,
};
use super::{rgb, ColorSpec};

pub use super::catalog_store::leak_str;

/// 找不到资源包时使用的兜底场景方块 string id
pub const FALLBACK_SCENE_STRING_ID: &str = "fallback";

/// 场景方块句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct SceneBlockId(pub u16);

impl From<u16> for SceneBlockId {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<SceneBlockId> for u16 {
    fn from(id: SceneBlockId) -> Self {
        id.0
    }
}

/// 单种场景方块的模拟侧定义
#[derive(Clone, Debug)]
pub struct SceneBlockDef {
    pub string_id: String,
    /// i18n key，形如 `block.grass`（泄漏为 `'static` 供旧 API）
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
    pub collision: bool,
    pub connectable: [bool; 6],
    /// 是否可朝向（放置时 R 旋转，存档持久化 facing）
    pub directional: bool,
    pub color: ColorSpec,
}

impl HasStringId for SceneBlockDef {
    fn string_id(&self) -> &str {
        &self.string_id
    }
}

impl CatalogEntry for SceneBlockDef {
    const LABEL: &'static str = "scene block";
}

/// 场景方块目录
pub type SceneBlockCatalog = StringIdCatalog<SceneBlockId, SceneBlockDef>;

impl SceneBlockCatalog {
    /// 解析 string id；未知则返回兜底场景方块
    pub fn resolve_id(&self, string_id: &str) -> SceneBlockId {
        self.id_by_string(string_id)
            .unwrap_or_else(|| self.fallback_id())
    }

    /// 目录内兜底场景方块（必存在）
    pub fn fallback_id(&self) -> SceneBlockId {
        self.id_by_string(FALLBACK_SCENE_STRING_ID)
            .or_else(|| (!self.is_empty()).then_some(SceneBlockId(0)))
            .expect("scene catalog must contain fallback")
    }
}

static CATALOG: LazyLock<RwLock<SceneBlockCatalog>> =
    LazyLock::new(|| RwLock::new(SceneBlockCatalog::new()));

/// 确保目录里有醒目的兜底场景方块
fn ensure_fallback_entry(catalog: &mut SceneBlockCatalog) {
    if catalog.id_by_string(FALLBACK_SCENE_STRING_ID).is_some() {
        return;
    }
    catalog
        .register(SceneBlockDef {
            string_id: FALLBACK_SCENE_STRING_ID.to_string(),
            name_key: leak_str("block.fallback"),
            short_name_key: leak_str("short.fallback"),
            description_key: leak_str("desc.fallback"),
            collision: true,
            connectable: [true; 6],
            directional: false,
            // UI 无贴图时的色块提示（洋红）
            color: rgb(1.0, 0.0, 1.0),
        })
        .expect("fallback scene id unique");
}

/// 安装/替换全局场景目录；保证兜底方块存在
pub fn install_scene_catalog(mut catalog: SceneBlockCatalog) {
    ensure_fallback_entry(&mut catalog);
    install_global(&CATALOG, catalog);
}

/// 读取当前目录快照
pub fn scene_catalog() -> SceneBlockCatalog {
    ensure_fallback_scene_catalog();
    clone_global(&CATALOG)
}

/// 当前兜底场景方块 id
pub fn fallback_scene_id() -> SceneBlockId {
    ensure_fallback_scene_catalog();
    with_global(&CATALOG, |c| c.fallback_id())
}

/// 按 string id 解析；未知 → 兜底
pub fn resolve_scene_id(string_id: &str) -> SceneBlockId {
    ensure_fallback_scene_catalog();
    with_global(&CATALOG, |c| c.resolve_id(string_id))
}

/// 按 id 查定义；未知 id → 兜底定义
pub fn scene_def(id: SceneBlockId) -> SceneBlockDef {
    ensure_fallback_scene_catalog();
    with_global(&CATALOG, |catalog| {
        catalog.get(id).cloned().unwrap_or_else(|| {
            catalog
                .get(catalog.fallback_id())
                .cloned()
                .expect("fallback def")
        })
    })
}

/// 确保兜底场景方块已注册
pub fn ensure_fallback_scene_catalog() {
    if with_global(&CATALOG, |c| c.id_by_string(FALLBACK_SCENE_STRING_ID).is_some()) {
        return;
    }
    with_global_mut(&CATALOG, ensure_fallback_entry);
}

