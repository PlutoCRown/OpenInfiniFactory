//! 材料 / 印花 / 滚刷表现注册表（贴图与可选模型路径）

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;

use crate::game::blocks::{
    ColorSpec, MaterialBlockId, PaintMaterialId, StampMaterialId,
};

/// 单个材料方块的表现数据
#[derive(Clone, Debug)]
pub struct MaterialBlockPresentation {
    pub id: MaterialBlockId,
    pub string_id: String,
    pub model_path: Option<PathBuf>,
    pub texture_path: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub color: ColorSpec,
}

/// 单个印花材料的表现数据
#[derive(Clone, Debug)]
pub struct StampMaterialPresentation {
    pub id: StampMaterialId,
    pub string_id: String,
    pub model_path: Option<PathBuf>,
    pub texture_path: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub color: ColorSpec,
}

/// 单个滚刷漆材料的表现数据（贴图必填）
#[derive(Clone, Debug)]
pub struct PaintMaterialPresentation {
    pub id: PaintMaterialId,
    pub string_id: String,
    pub texture_path: PathBuf,
}

/// 游戏侧材料方块注册表
#[derive(Resource, Clone, Debug, Default)]
pub struct MaterialBlockRegistry {
    by_id: HashMap<MaterialBlockId, MaterialBlockPresentation>,
    order: Vec<MaterialBlockId>,
}

impl MaterialBlockRegistry {
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.order.clear();
    }

    pub fn insert(&mut self, presentation: MaterialBlockPresentation) {
        let id = presentation.id;
        if !self.by_id.contains_key(&id) {
            self.order.push(id);
        }
        self.by_id.insert(id, presentation);
    }

    pub fn get(&self, id: MaterialBlockId) -> Option<&MaterialBlockPresentation> {
        self.by_id.get(&id)
    }

    pub fn ordered(&self) -> impl Iterator<Item = &MaterialBlockPresentation> {
        self.order.iter().filter_map(|id| self.by_id.get(id))
    }
}

/// 游戏侧印花材料注册表
#[derive(Resource, Clone, Debug, Default)]
pub struct StampMaterialRegistry {
    by_id: HashMap<StampMaterialId, StampMaterialPresentation>,
    order: Vec<StampMaterialId>,
}

impl StampMaterialRegistry {
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.order.clear();
    }

    pub fn insert(&mut self, presentation: StampMaterialPresentation) {
        let id = presentation.id;
        if !self.by_id.contains_key(&id) {
            self.order.push(id);
        }
        self.by_id.insert(id, presentation);
    }

    pub fn get(&self, id: StampMaterialId) -> Option<&StampMaterialPresentation> {
        self.by_id.get(&id)
    }

    pub fn ordered(&self) -> impl Iterator<Item = &StampMaterialPresentation> {
        self.order.iter().filter_map(|id| self.by_id.get(id))
    }
}

/// 游戏侧滚刷漆材料注册表
#[derive(Resource, Clone, Debug, Default)]
pub struct PaintMaterialRegistry {
    by_id: HashMap<PaintMaterialId, PaintMaterialPresentation>,
    order: Vec<PaintMaterialId>,
}

impl PaintMaterialRegistry {
    pub fn clear(&mut self) {
        self.by_id.clear();
        self.order.clear();
    }

    pub fn insert(&mut self, presentation: PaintMaterialPresentation) {
        let id = presentation.id;
        if !self.by_id.contains_key(&id) {
            self.order.push(id);
        }
        self.by_id.insert(id, presentation);
    }

    pub fn get(&self, id: PaintMaterialId) -> Option<&PaintMaterialPresentation> {
        self.by_id.get(&id)
    }

    pub fn ordered(&self) -> impl Iterator<Item = &PaintMaterialPresentation> {
        self.order.iter().filter_map(|id| self.by_id.get(id))
    }
}
