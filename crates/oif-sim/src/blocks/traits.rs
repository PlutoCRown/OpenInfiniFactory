use glam::IVec3;

use crate::world::direction::Facing;
use crate::world::grid::BlockSettings;

use super::{
    BlockDefinition, BlockKind, LaserOpticsBehavior, MarkerBehavior, MaterialDestroyer,
    MaterialLabeler, MaterialProcessor, MaterialSource, MovementRule, PersistentLayer,
    SignalBehavior, WeldBehavior,
};

/// 方块身份与目录元数据、实例默认设置
pub trait BlockMeta: Send + Sync {
    fn id(&self) -> BlockKind;
    fn definition(&self) -> BlockDefinition;

    fn alternate(&self) -> Option<BlockKind> {
        None
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
}
