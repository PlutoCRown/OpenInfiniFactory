//! 调试入口与模拟日志 re-export

#[cfg(not(target_arch = "wasm32"))]
pub use crate::debug_http::embedded::DebugToolsPlugin;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::debug_http::snapshot::target_status_line;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::debug_http::{
    DebugHttpBridge, PendingDebugHttpStart, poll_debug_http, start_debug_http_server,
    try_start_debug_http_server,
};

pub use oif_sim::SimulationDebugLog;
