//! 无头模拟会话：自有状态 + 控制面（不依赖 Bevy App）

pub mod control;
pub mod log;
pub mod session;

pub use control::SimulationControl;
pub use log::SimulationDebugLog;
pub use session::SimSession;
