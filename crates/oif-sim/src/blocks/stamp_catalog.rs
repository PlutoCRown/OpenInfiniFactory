//! 印花材料运行时目录：字符串 id ↔ StampMaterialId，以及 fragile/color 等模拟元数据

use std::collections::HashMap;
use std::sync::{LazyLock, Once, RwLock};

use serde::{Deserialize, Serialize};

use super::scene_catalog::leak_str;
use super::{rgb, ColorSpec};

/// 印花材料句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StampMaterialId(pub u16);

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

/// 印花材料目录
#[derive(Clone, Debug, Default)]
pub struct StampMaterialCatalog {
    defs: Vec<StampMaterialDef>,
    by_string: HashMap<String, StampMaterialId>,
}

impl StampMaterialCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn get(&self, id: StampMaterialId) -> Option<&StampMaterialDef> {
        self.defs.get(id.0 as usize)
    }

    pub fn id_by_string(&self, string_id: &str) -> Option<StampMaterialId> {
        self.by_string.get(string_id).copied()
    }

    pub fn string_id(&self, id: StampMaterialId) -> Option<&str> {
        self.defs.get(id.0 as usize).map(|d| d.string_id.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (StampMaterialId, &StampMaterialDef)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, def)| (StampMaterialId(i as u16), def))
    }

    /// 注册一种印花材料；`string_id` 重复则返回 Err
    pub fn register(&mut self, def: StampMaterialDef) -> Result<StampMaterialId, String> {
        if self.by_string.contains_key(&def.string_id) {
            return Err(format!("duplicate stamp material id '{}'", def.string_id));
        }
        if self.defs.len() >= u16::MAX as usize {
            return Err("stamp material catalog full".into());
        }
        let id = StampMaterialId(self.defs.len() as u16);
        self.by_string.insert(def.string_id.clone(), id);
        self.defs.push(def);
        Ok(id)
    }
}

static CATALOG: LazyLock<RwLock<StampMaterialCatalog>> =
    LazyLock::new(|| RwLock::new(StampMaterialCatalog::new()));

static FALLBACK_ONCE: Once = Once::new();

/// 安装/替换全局印花目录（游戏加载资源包时调用）
pub fn install_stamp_catalog(catalog: StampMaterialCatalog) {
    *CATALOG.write().expect("stamp catalog lock") = catalog;
}

/// 读取当前目录快照
pub fn stamp_catalog() -> StampMaterialCatalog {
    ensure_fallback_stamp_catalog();
    CATALOG.read().expect("stamp catalog lock").clone()
}

/// 按 id 查定义（无则 panic，用于已解析的 BlockKind::Stamp）
pub fn stamp_def(id: StampMaterialId) -> StampMaterialDef {
    ensure_fallback_stamp_catalog();
    CATALOG
        .read()
        .expect("stamp catalog lock")
        .get(id)
        .cloned()
        .unwrap_or_else(|| panic!("unknown StampMaterialId {}", id.0))
}

/// 无资源包时的兜底（单测 / wasm）：注册红绿蓝黄，编号不保证跨会话稳定
pub fn ensure_fallback_stamp_catalog() {
    FALLBACK_ONCE.call_once(|| {
        let mut catalog = CATALOG.write().expect("stamp catalog lock");
        if !catalog.is_empty() {
            return;
        }
        let entries: [(&str, ColorSpec); 4] = [
            ("red", rgb(242.0 / 255.0, 31.0 / 255.0, 26.0 / 255.0)),
            ("green", rgb(51.0 / 255.0, 209.0 / 255.0, 71.0 / 255.0)),
            ("blue", rgb(46.0 / 255.0, 107.0 / 255.0, 242.0 / 255.0)),
            ("yellow", rgb(255.0 / 255.0, 214.0 / 255.0, 46.0 / 255.0)),
        ];
        for (string_id, color) in entries {
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
}
