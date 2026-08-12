//! 模拟类型兼容 facade：直接复用 `oif_sim` 的权威 ECS Resource

pub use oif_sim::simulation::{BreakDebris, LaserBeam, LaserBeamStop};

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
    pub use oif_sim::simulation::pending::*;
}

pub mod signals {
    pub use oif_sim::simulation::signals::*;
}

pub mod stats {
    pub use oif_sim::simulation::stats::*;
}

pub mod structure_state {
    pub use oif_sim::simulation::structure_state::*;
}

pub mod structures {
    pub use oif_sim::simulation::structures::*;
}

pub mod movement {
    pub use oif_sim::simulation::movement::*;
}
