//! 材料方块运行时目录：字符串 id ↔ MaterialBlockId，以及 connectable/fragile 等模拟元数据

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use super::scene_catalog::leak_str;
use super::{ColorSpec, rgb};

/// 找不到资源包时使用的兜底材料 string id
pub const FALLBACK_MATERIAL_STRING_ID: &str = "fallback";

/// 材料方块句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct MaterialBlockId(pub u16);

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

/// 材料方块目录
#[derive(Clone, Debug, Default)]
pub struct MaterialBlockCatalog {
    defs: Vec<MaterialBlockDef>,
    by_string: HashMap<String, MaterialBlockId>,
}

impl MaterialBlockCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn get(&self, id: MaterialBlockId) -> Option<&MaterialBlockDef> {
        self.defs.get(id.0 as usize)
    }

    pub fn id_by_string(&self, string_id: &str) -> Option<MaterialBlockId> {
        self.by_string.get(string_id).copied()
    }

    /// 解析 string id；未知则返回兜底材料
    pub fn resolve_id(&self, string_id: &str) -> MaterialBlockId {
        self.id_by_string(string_id)
            .unwrap_or_else(|| self.fallback_id())
    }

    /// 目录内兜底材料（必存在）
    pub fn fallback_id(&self) -> MaterialBlockId {
        self.id_by_string(FALLBACK_MATERIAL_STRING_ID)
            .or_else(|| self.defs.first().map(|_| MaterialBlockId(0)))
            .expect("material catalog must contain fallback")
    }

    pub fn string_id(&self, id: MaterialBlockId) -> Option<&str> {
        self.defs.get(id.0 as usize).map(|d| d.string_id.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (MaterialBlockId, &MaterialBlockDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, def)| (MaterialBlockId(i as u16), def))
    }

    /// 注册一种材料方块；`string_id` 重复则返回 Err
    pub fn register(&mut self, def: MaterialBlockDef) -> Result<MaterialBlockId, String> {
        if self.by_string.contains_key(&def.string_id) {
            return Err(format!("duplicate material block id '{}'", def.string_id));
        }
        if self.defs.len() >= u16::MAX as usize {
            return Err("material block catalog full".into());
        }
        let id = MaterialBlockId(self.defs.len() as u16);
        self.by_string.insert(def.string_id.clone(), id);
        self.defs.push(def);
        Ok(id)
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
    *CATALOG.write().expect("material catalog lock") = catalog;
}

/// 读取当前目录快照
pub fn material_catalog() -> MaterialBlockCatalog {
    ensure_fallback_material_catalog();
    CATALOG.read().expect("material catalog lock").clone()
}

/// 当前兜底材料 id
pub fn fallback_material_id() -> MaterialBlockId {
    material_catalog().fallback_id()
}

/// 按 string id 解析；未知 → 兜底
pub fn resolve_material_id(string_id: &str) -> MaterialBlockId {
    material_catalog().resolve_id(string_id)
}
/// 按 id 查定义；未知 id → 兜底定义
pub fn material_def(id: MaterialBlockId) -> MaterialBlockDef {
    ensure_fallback_material_catalog();
    let catalog = CATALOG.read().expect("material catalog lock");
    catalog
        .get(id)
        .cloned()
        .unwrap_or_else(|| catalog.get(catalog.fallback_id()).cloned().expect("fallback def"))
}

/// 确保兜底材料已注册
pub fn ensure_fallback_material_catalog() {
    if CATALOG
        .read()
        .expect("material catalog lock")
        .id_by_string(FALLBACK_MATERIAL_STRING_ID)
        .is_some()
    {
        return;
    }
    let mut catalog = CATALOG.write().expect("material catalog lock");
    ensure_fallback_entry(&mut catalog);
}
