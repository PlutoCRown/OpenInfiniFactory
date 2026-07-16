//! 模拟逻辑已迁至 `oif_sim`；此处提供 Bevy Resource 包装与路径兼容 re-export

pub use oif_sim::simulation::{LaserBeam, LaserBeamStop};

pub mod core {
    pub use oif_sim::simulation::core::*;
}

pub mod markers {
    pub use oif_sim::simulation::markers::*;
}

pub mod motion {
    pub use oif_sim::simulation::motion::*;
}

pub mod pending {
    use bevy::prelude::*;

    /// 跨回合挂起的生成材料（Bevy Resource）
    #[derive(Resource, Deref, DerefMut, Clone, Default)]
    pub struct PendingGeneratedMaterials(pub oif_sim::simulation::pending::PendingGeneratedMaterials);
}

pub mod signals {
    pub use oif_sim::simulation::signals::SignalComponentId;
    use bevy::prelude::*;

    /// 信号网络缓存（Bevy Resource）
    #[derive(Resource, Deref, DerefMut, Clone, Default)]
    pub struct SignalNetworkCache(pub oif_sim::simulation::signals::SignalNetworkCache);
}

pub mod stats {
    use bevy::prelude::*;

    /// 单回合模拟各阶段耗时采样（Bevy Resource）
    #[derive(Resource, Deref, DerefMut, Clone, Default)]
    pub struct SimulationStepStats(pub oif_sim::simulation::stats::SimulationStepStats);
}

pub mod structure_state {
    pub use oif_sim::simulation::structure_state::{
        material_structure, query_factory_structure, FactoryActivity, StructureFreedom,
        StructureId, StructureKind,
    };
    use bevy::prelude::*;

    /// 结构运行时状态（Bevy Resource）
    #[derive(Resource, Deref, DerefMut, Clone, Default)]
    pub struct StructureState(pub oif_sim::simulation::structure_state::StructureState);
}

pub mod structures {
    use bevy::prelude::*;

    /// 运动影响缓存（Bevy Resource）
    #[derive(Resource, Deref, DerefMut, Clone, Default)]
    pub struct MovementInfluenceCache(pub oif_sim::simulation::structures::MovementInfluenceCache);
}

pub mod movement {
    use bevy::prelude::*;

    use crate::game::world::grid::WorldBlocks;

    /// 推杆伸出状态（Bevy Resource）
    #[derive(Resource, Deref, DerefMut, Clone, Default)]
    pub struct PusherState(pub oif_sim::simulation::movement::PusherState);

    impl PusherState {
        /// 从世界重建推杆状态
        pub fn rebuild_from_world(world: &WorldBlocks) -> Self {
            Self(oif_sim::simulation::movement::PusherState::rebuild_from_world(world))
        }
    }
}
