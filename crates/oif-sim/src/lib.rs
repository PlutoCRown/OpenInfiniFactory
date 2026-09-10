//! OpenInfiniFactory 模拟核心：世界、方块声明、回合模拟与无头会话

pub mod blocks;
pub mod session;
pub mod simulation;
pub mod world;

pub use blocks::{
    BlockData, BlockId, BlockKind, MaterialBlockId, MaterialProps, PaintMaterialId, StampMaterialId,
};
pub use session::{SimSession, SimulationControl, SimulationDebugLog};
pub use simulation::core::{TurnOutput, simulate_turn};
pub use world::{Facing, WorldBlocks};
