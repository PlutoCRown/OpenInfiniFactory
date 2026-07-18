//! 场景方块运行时目录：字符串 id ↔ SceneBlockId，以及碰撞/connectable 等模拟元数据

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use super::{ColorSpec, rgb};

/// 找不到资源包时使用的兜底场景方块 string id
pub const FALLBACK_SCENE_STRING_ID: &str = "fallback";

/// 场景方块句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct SceneBlockId(pub u16);

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

/// 场景方块目录
#[derive(Clone, Debug, Default)]
pub struct SceneBlockCatalog {
    defs: Vec<SceneBlockDef>,
    by_string: HashMap<String, SceneBlockId>,
}

impl SceneBlockCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn get(&self, id: SceneBlockId) -> Option<&SceneBlockDef> {
        self.defs.get(id.0 as usize)
    }

    pub fn id_by_string(&self, string_id: &str) -> Option<SceneBlockId> {
        self.by_string.get(string_id).copied()
    }

    /// 解析 string id；未知则返回兜底场景方块
    pub fn resolve_id(&self, string_id: &str) -> SceneBlockId {
        self.id_by_string(string_id)
            .unwrap_or_else(|| self.fallback_id())
    }

    /// 目录内兜底场景方块（必存在）
    pub fn fallback_id(&self) -> SceneBlockId {
        self.id_by_string(FALLBACK_SCENE_STRING_ID)
            .or_else(|| self.defs.first().map(|_| SceneBlockId(0)))
            .expect("scene catalog must contain fallback")
    }

    pub fn string_id(&self, id: SceneBlockId) -> Option<&str> {
        self.defs.get(id.0 as usize).map(|d| d.string_id.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SceneBlockId, &SceneBlockDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, def)| (SceneBlockId(i as u16), def))
    }

    /// 注册一种场景方块；`string_id` 重复则返回 Err
    pub fn register(&mut self, def: SceneBlockDef) -> Result<SceneBlockId, String> {
        if self.by_string.contains_key(&def.string_id) {
            return Err(format!("duplicate scene block id '{}'", def.string_id));
        }
        if self.defs.len() >= u16::MAX as usize {
            return Err("scene block catalog full".into());
        }
        let id = SceneBlockId(self.defs.len() as u16);
        self.by_string.insert(def.string_id.clone(), id);
        self.defs.push(def);
        Ok(id)
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
    *CATALOG.write().expect("scene catalog lock") = catalog;
}

/// 读取当前目录快照
pub fn scene_catalog() -> SceneBlockCatalog {
    ensure_fallback_scene_catalog();
    CATALOG.read().expect("scene catalog lock").clone()
}

/// 当前兜底场景方块 id
pub fn fallback_scene_id() -> SceneBlockId {
    scene_catalog().fallback_id()
}

/// 按 string id 解析；未知 → 兜底
pub fn resolve_scene_id(string_id: &str) -> SceneBlockId {
    scene_catalog().resolve_id(string_id)
}

/// 按 id 查定义；未知 id → 兜底定义
pub fn scene_def(id: SceneBlockId) -> SceneBlockDef {
    ensure_fallback_scene_catalog();
    let catalog = CATALOG.read().expect("scene catalog lock");
    catalog.get(id).cloned().unwrap_or_else(|| {
        catalog
            .get(catalog.fallback_id())
            .cloned()
            .expect("fallback def")
    })
}

/// 确保兜底场景方块已注册
pub fn ensure_fallback_scene_catalog() {
    if CATALOG
        .read()
        .expect("scene catalog lock")
        .id_by_string(FALLBACK_SCENE_STRING_ID)
        .is_some()
    {
        return;
    }
    let mut catalog = CATALOG.write().expect("scene catalog lock");
    ensure_fallback_entry(&mut catalog);
}

/// 把字符串泄漏为 `'static`（资源包 id / i18n key）
pub fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}
