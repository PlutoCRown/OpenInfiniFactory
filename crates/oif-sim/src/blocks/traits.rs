use glam::IVec3;

use crate::world::direction::Facing;
use crate::world::grid::BlockSettings;

use super::{
    BlockDefinition, BlockKind, LaserOpticsBehavior, MarkerBehavior, MaterialDestroyer,
    MaterialLabeler, MaterialProcessor, MaterialSource, MovementRule, PersistentLayer,
    PoweredSideEffect, SignalBehavior, WeldBehavior,
};

/// 方块身份与目录元数据、实例默认设置
pub trait BlockMeta: Send + Sync {
    fn id(&self) -> BlockKind;
    fn definition(&self) -> BlockDefinition;

    fn alternate(&self) -> Option<BlockKind> {
        None
    }

    /// 切换到 alternate 时是否把朝向转 180°（如传送带正反）
    fn alternate_flip_facing(&self) -> bool {
        false
    }

    fn persistent_layer(&self) -> Option<PersistentLayer> {
        self.definition().persistence
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        None
    }
}

/// 模拟侧行为声明：运动、信号、标记等（Trait 填表）
pub trait BlockBehavior: Send + Sync {
    fn is_directional(&self) -> bool {
        false
    }

    fn non_connection_face(&self, _facing: Facing) -> Option<IVec3> {
        None
    }

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        None
    }

    fn material_source(&self, _facing: Facing) -> Option<MaterialSource> {
        None
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        None
    }

    fn material_destroyer(&self, _facing: Facing) -> Option<MaterialDestroyer> {
        None
    }

    fn material_labeler(&self, _facing: Facing) -> Option<MaterialLabeler> {
        None
    }

    fn material_processor(&self) -> Option<MaterialProcessor> {
        None
    }

    fn laser_optics(&self) -> Option<LaserOpticsBehavior> {
        None
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        None
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        None
    }

    fn powered_side_effect(&self) -> Option<PoweredSideEffect> {
        None
    }

    /// 是否可作为方块传感器的检测目标（材料层另见 `BlockKind::is_detector_target`）
    fn is_detector_target(&self) -> bool {
        false
    }

    /// 机身格是否允许印花材料沿工作朝向透传进入
    fn allows_stamp_passthrough(&self) -> bool {
        false
    }

    /// 是否验收材料（Goal 等）
    fn accepts_material(&self) -> bool {
        false
    }

    /// 是否展示材料外壳预览（Goal / Generator 等）
    fn shows_material_preview(&self) -> bool {
        false
    }

    /// 是否贴工厂面放置并建立 factory_attachments（告示等）
    fn attaches_to_factory_face(&self) -> bool {
        false
    }
}
