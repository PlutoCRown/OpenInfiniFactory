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
pub mod counter_rotator;
pub mod detector;
pub mod down_detector;
pub mod down_welder;
pub mod drill;
pub mod drill_head;
pub mod generator;
pub mod goal;
pub mod laser;
pub mod lifter;
pub mod mirror;
pub mod platform;
pub mod pusher;
pub mod reverse_conveyor;
pub mod roller;
pub mod roller_body;
pub mod rotator;
pub mod splitter;
pub mod stamper;
pub mod stamper_body;
pub mod sign;
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
    ensure_fallback_material_catalog, ensure_fallback_paint_catalog, ensure_fallback_scene_catalog,
    ensure_fallback_stamp_catalog, fallback_material_id, fallback_scene_id,
    install_material_catalog, install_paint_catalog, install_scene_catalog, install_stamp_catalog,
    leak_str, material_catalog, material_def, paint_catalog, paint_def, resolve_material_id,
    resolve_scene_id, rgb, rgba, scene_catalog, scene_def, stamp_catalog, stamp_def, AcceptorId,
    BlockClass, BlockData, BlockDefinition, BlockId, BlockKind, BlockLayer, BlockShape, ColorSpec,
    Facing, FactoryBlock, LaserOpticsBehavior, MarkerBehavior, MaterialBlock, MaterialBlockCatalog,
    MaterialBlockDef, MaterialBlockId, MaterialDestroyer, MaterialLabeler, MaterialProcessor,
    MaterialProps, MaterialSource, MovementRule, PaintMaterialCatalog, PaintMaterialDef,
    PaintMaterialId, PersistentLayer, SceneBlockCatalog, SceneBlockDef, SceneBlockId,
    SignalBehavior, StampMaterialCatalog, StampMaterialDef, StampMaterialId, SystemBlock,
    VirtualBlock, WeldBehavior, BLOCK_SIZE, DEFAULT_GENERATOR_PERIOD, FALLBACK_MATERIAL_STRING_ID,
    FALLBACK_SCENE_STRING_ID,
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
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return self.material();
        }
        registry::placeable(self)
            .expect("inventory blocks must implement PlaceableBlock")
            .item_slot_color()
    }

    fn is_editable(self) -> bool {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return matches!(self, BlockKind::Scene(_));
        }
        registry::is_editable(self)
    }

    fn ui_panel(self) -> Option<UiPanelId> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        registry::editable(self).and_then(traits::BlockUi::ui_panel)
    }

    fn render_behavior(self, facing: Facing) -> RenderBehavior {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return RenderBehavior::default();
        }
        registry::get(self).render_behavior(facing)
    }

    fn model(self) -> BlockModel {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return BlockModel::Default;
        }
        registry::get(self).model()
    }

    fn block_texture(self) -> Option<Image> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        registry::get(self).block_texture()
    }
}
