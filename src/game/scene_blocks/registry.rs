//! 场景方块表现注册表（模型路径、可选碰撞模型）

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;

use crate::game::blocks::{scene_catalog, BlockKind, ColorSpec, SceneBlockId};

/// 单个场景方块的表现数据（外观以 model.glb 为准）
#[derive(Clone, Debug)]
pub struct SceneBlockPresentation {
    pub id: SceneBlockId,
    pub string_id: String,
    pub model_path: PathBuf,
    pub collision_model_path: Option<PathBuf>,
    /// 预烘焙 UI 图标（同目录 icon.png）；缺省则热键栏无图
    pub icon_path: Option<PathBuf>,
    pub color: ColorSpec,
}

/// 游戏侧场景方块注册表
#[derive(Resource, Clone, Debug, Default)]
pub struct SceneBlockRegistry {
    by_id: HashMap<SceneBlockId, SceneBlockPresentation>,
    /// 按注册顺序，供编辑热键栏
    order: Vec<SceneBlockId>,
}

impl SceneBlockRegistry {
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.order.clear();
    }

    pub fn insert(&mut self, presentation: SceneBlockPresentation) {
        let id = presentation.id;
        if !self.by_id.contains_key(&id) {
            self.order.push(id);
        }
        self.by_id.insert(id, presentation);
    }

    pub fn get(&self, id: SceneBlockId) -> Option<&SceneBlockPresentation> {
        self.by_id.get(&id)
    }

    pub fn get_kind(&self, kind: BlockKind) -> Option<&SceneBlockPresentation> {
        match kind {
            BlockKind::Scene(id) => self.get(id),
            _ => None,
        }
    }

    pub fn ordered_kinds(&self) -> Vec<BlockKind> {
        self.order.iter().copied().map(BlockKind::Scene).collect()
    }

    pub fn string_id(&self, id: SceneBlockId) -> Option<&str> {
        self.by_id.get(&id).map(|p| p.string_id.as_str())
    }

    /// 与模拟 catalog 对齐的展示色
    pub fn color(&self, id: SceneBlockId) -> ColorSpec {
        self.by_id
            .get(&id)
            .map(|p| p.color)
            .or_else(|| scene_catalog().get(id).map(|d| d.color))
            .unwrap_or(ColorSpec {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            })
    }
}
