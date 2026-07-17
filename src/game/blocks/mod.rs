//! 方块表现层：在 `oif_sim` 类型上挂载 Render / Ui / Placeable，并注册游戏侧 inventory

mod adapter;
pub mod panels;
#[macro_use]
mod register;
mod registry;
pub mod render_types;
pub mod traits;

pub mod blocker;
pub mod converter;
pub mod conveyor;
pub mod copper_material;
pub mod counter_rotator;
pub mod detector;
pub mod dirt;
pub mod down_detector;
pub mod down_welder;
pub mod drill;
pub mod drill_head;
pub mod generator;
pub mod glass_material;
pub mod goal;
pub mod grass;
pub mod iron_material;
pub mod laser;
pub mod lifter;
pub mod material;
pub mod mirror;
pub mod planks;
pub mod platform;
pub mod pusher;
pub mod reverse_conveyor;
pub mod roller;
pub mod roller_body;
pub mod rotator;
pub mod splitter;
pub mod stamper;
pub mod stamper_body;
pub mod stone;
pub mod suction_cup;
pub mod teleport_entrance;
pub mod teleport_exit;
pub mod vertical_mirror;
pub mod weld_point;
pub mod welder;
pub mod wire;

mod model_spawn;

use bevy::prelude::*;

pub use self::model_spawn::spawn_model_parts;
pub use self::registry::{
    all_blocks, assert_registry_consistent, edit_blocks, save_stores_facing, PLAY_BLOCKS,
};
pub use self::render_types::{
    render_directional_wire_device, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
    RenderBehavior, WeldConnectorBehavior, WireConnectorBehavior,
};
pub use oif_sim::blocks::{
    rgb, rgba, AcceptorId, BlockClass, BlockData, BlockDefinition, BlockId, BlockKind, BlockLayer,
    BlockShape, ColorSpec, Facing, FactoryBlock, LaserOpticsBehavior, MarkerBehavior,
    MaterialBlock, MaterialDestroyer, MaterialKind, MaterialLabeler, MaterialProcessor,
    MaterialProps, MaterialSource, MovementRule, PersistentLayer, SceneBlock, SignalBehavior,
    StampColor, SystemBlock, VirtualBlock, WeldBehavior, BLOCK_SIZE, DEFAULT_GENERATOR_PERIOD,
};
use crate::game::state::UiPanelId;

/// ColorSpec → Bevy Color
pub trait ColorSpecExt {
    fn color(self) -> Color;
}

impl ColorSpecExt for ColorSpec {
    fn color(self) -> Color {
        Color::srgba(self.r, self.g, self.b, self.a)
    }
}

/// 游戏侧完整方块：模拟 Meta/Behavior + 表现 Render
pub trait Block:
    oif_sim::blocks::traits::BlockMeta + oif_sim::blocks::traits::BlockBehavior + traits::BlockRender
{
}

impl<T> Block for T where
    T: oif_sim::blocks::traits::BlockMeta
        + oif_sim::blocks::traits::BlockBehavior
        + traits::BlockRender
{
}

/// 可打开属性面板的方块
pub trait EditableBlock: Block + traits::BlockUi {}

impl<T> EditableBlock for T where T: Block + traits::BlockUi {}

/// 表现层扩展：在 `oif_sim::BlockKind` 上查询 Render / Ui / Placeable
pub trait BlockPresent: Sized {
    fn material(self) -> Color;
    fn item_slot_color(self) -> Color;
    fn is_editable(self) -> bool;
    fn ui_panel(self) -> Option<UiPanelId>;
    fn render_behavior(self, facing: Facing) -> RenderBehavior;
    fn model(self) -> BlockModel;
    fn block_texture(self) -> Option<Image>;
}

impl BlockPresent for BlockKind {
    fn material(self) -> Color {
        self.definition().color().color()
    }

    fn item_slot_color(self) -> Color {
        registry::placeable(self)
            .expect("inventory blocks must implement PlaceableBlock")
            .item_slot_color()
    }

    fn is_editable(self) -> bool {
        registry::is_editable(self)
    }

    fn ui_panel(self) -> Option<UiPanelId> {
        registry::editable(self).and_then(traits::BlockUi::ui_panel)
    }

    fn render_behavior(self, facing: Facing) -> RenderBehavior {
        registry::get(self).render_behavior(facing)
    }

    fn model(self) -> BlockModel {
        registry::get(self).model()
    }

    fn block_texture(self) -> Option<Image> {
        registry::get(self).block_texture()
    }
}
