//! 滚刷漆材料运行时目录：字符串 id ↔ PaintMaterialId

use std::collections::HashMap;
use std::sync::{LazyLock, Once, RwLock};

use serde::{Deserialize, Serialize};

use super::scene_catalog::leak_str;

/// 滚刷漆材料句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PaintMaterialId(pub u16);

/// 单种滚刷漆材料的模拟侧定义
#[derive(Clone, Debug)]
pub struct PaintMaterialDef {
    pub string_id: String,
    /// i18n key，形如 `paint.red`（泄漏为 `'static` 供旧 API）
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
}

/// 滚刷漆材料目录
#[derive(Clone, Debug, Default)]
pub struct PaintMaterialCatalog {
    defs: Vec<PaintMaterialDef>,
    by_string: HashMap<String, PaintMaterialId>,
}

impl PaintMaterialCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn get(&self, id: PaintMaterialId) -> Option<&PaintMaterialDef> {
        self.defs.get(id.0 as usize)
    }

    pub fn id_by_string(&self, string_id: &str) -> Option<PaintMaterialId> {
        self.by_string.get(string_id).copied()
    }

    pub fn string_id(&self, id: PaintMaterialId) -> Option<&str> {
        self.defs.get(id.0 as usize).map(|d| d.string_id.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (PaintMaterialId, &PaintMaterialDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, def)| (PaintMaterialId(i as u16), def))
    }

    /// 注册一种滚刷漆；`string_id` 重复则返回 Err
    pub fn register(&mut self, def: PaintMaterialDef) -> Result<PaintMaterialId, String> {
        if self.by_string.contains_key(&def.string_id) {
            return Err(format!("duplicate paint material id '{}'", def.string_id));
        }
        if self.defs.len() >= u16::MAX as usize {
            return Err("paint material catalog full".into());
        }
        let id = PaintMaterialId(self.defs.len() as u16);
        self.by_string.insert(def.string_id.clone(), id);
        self.defs.push(def);
        Ok(id)
    }
}

static CATALOG: LazyLock<RwLock<PaintMaterialCatalog>> =
    LazyLock::new(|| RwLock::new(PaintMaterialCatalog::new()));

static FALLBACK_ONCE: Once = Once::new();

/// 安装/替换全局滚刷漆目录（游戏加载资源包时调用）
pub fn install_paint_catalog(catalog: PaintMaterialCatalog) {
    *CATALOG.write().expect("paint catalog lock") = catalog;
}

/// 读取当前目录快照
pub fn paint_catalog() -> PaintMaterialCatalog {
    ensure_fallback_paint_catalog();
    CATALOG.read().expect("paint catalog lock").clone()
}

/// 按 id 查定义（无则 panic）
pub fn paint_def(id: PaintMaterialId) -> PaintMaterialDef {
    ensure_fallback_paint_catalog();
    CATALOG
        .read()
        .expect("paint catalog lock")
        .get(id)
        .cloned()
        .unwrap_or_else(|| panic!("unknown PaintMaterialId {}", id.0))
}

/// 无资源包时的兜底（单测 / wasm）：注册红绿蓝黄，编号不保证跨会话稳定
pub fn ensure_fallback_paint_catalog() {
    FALLBACK_ONCE.call_once(|| {
        let mut catalog = CATALOG.write().expect("paint catalog lock");
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
}
