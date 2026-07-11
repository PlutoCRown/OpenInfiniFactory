use bevy::prelude::*;

use crate::game::state::UiPanelId;
use crate::game::world::direction::Facing;

use super::{BlockModel, RenderBehavior};

/// 3D 模型与连接器渲染提示
pub trait BlockRender: Send + Sync {
    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior::default()
    }

    fn model(&self) -> BlockModel {
        BlockModel::Default
    }

    fn block_texture(&self) -> Option<Image> {
        None
    }
}

/// 物品栏槽位着色
pub trait PlaceableBlock: Send + Sync {
    fn item_slot_color(&self) -> Color;
}

/// 属性面板编辑
pub trait BlockUi: Send + Sync {
    fn ui_panel(&self) -> Option<UiPanelId>;
}
