//! 材料方块运行时目录：字符串 id ↔ MaterialBlockId，以及 connectable/fragile 等模拟元数据

use std::collections::HashMap;
use std::sync::{LazyLock, Once, RwLock};

use serde::{Deserialize, Serialize};

use super::scene_catalog::leak_str;
use super::{ColorSpec, rgb};

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

static FALLBACK_ONCE: Once = Once::new();

/// 安装/替换全局材料目录（游戏加载资源包时调用）
pub fn install_material_catalog(catalog: MaterialBlockCatalog) {
    *CATALOG.write().expect("material catalog lock") = catalog;
}

/// 读取当前目录快照
pub fn material_catalog() -> MaterialBlockCatalog {
    ensure_fallback_material_catalog();
    CATALOG.read().expect("material catalog lock").clone()
}

/// 按 id 查定义（无则 panic，用于已解析的 BlockKind::Material）
pub fn material_def(id: MaterialBlockId) -> MaterialBlockDef {
    ensure_fallback_material_catalog();
    CATALOG
        .read()
        .expect("material catalog lock")
        .get(id)
        .cloned()
        .unwrap_or_else(|| panic!("unknown MaterialBlockId {}", id.0))
}

/// 无资源包时的兜底（单测 / wasm）：注册最小可玩集合，编号不保证跨会话稳定
pub fn ensure_fallback_material_catalog() {
    FALLBACK_ONCE.call_once(|| {
        let mut catalog = CATALOG.write().expect("material catalog lock");
        if !catalog.is_empty() {
            return;
        }
        let entries: [(&str, bool, ColorSpec); 10] = [
            (
                "basic",
                false,
                rgb(214.0 / 255.0, 186.0 / 255.0, 118.0 / 255.0),
            ),
            (
                "iron",
                false,
                rgb(160.0 / 255.0, 168.0 / 255.0, 176.0 / 255.0),
            ),
            (
                "copper",
                false,
                rgb(200.0 / 255.0, 110.0 / 255.0, 58.0 / 255.0),
            ),
            (
                "glass_material",
                true,
                rgb(168.0 / 255.0, 214.0 / 255.0, 228.0 / 255.0),
            ),
            (
                "gold",
                false,
                rgb(232.0 / 255.0, 190.0 / 255.0, 70.0 / 255.0),
            ),
            (
                "aluminum",
                false,
                rgb(200.0 / 255.0, 208.0 / 255.0, 216.0 / 255.0),
            ),
            (
                "wood",
                false,
                rgb(150.0 / 255.0, 95.0 / 255.0, 48.0 / 255.0),
            ),
            (
                "granite",
                false,
                rgb(140.0 / 255.0, 142.0 / 255.0, 148.0 / 255.0),
            ),
            ("coal", false, rgb(36.0 / 255.0, 36.0 / 255.0, 40.0 / 255.0)),
            (
                "crystal",
                true,
                rgb(140.0 / 255.0, 120.0 / 255.0, 220.0 / 255.0),
            ),
        ];
        for (string_id, fragile, color) in entries {
            let name_key = leak_str(&format!("block.{string_id}"));
            let short_name_key = leak_str(&format!("short.{string_id}"));
            let description_key = leak_str(&format!("desc.{string_id}"));
            catalog
                .register(MaterialBlockDef {
                    string_id: string_id.to_string(),
                    name_key,
                    short_name_key,
                    description_key,
                    connectable: [true; 6],
                    fragile,
                    directional: false,
                    color,
                })
                .expect("fallback material id unique");
        }
    });
}
