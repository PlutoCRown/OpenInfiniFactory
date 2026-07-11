pub use crate::debug_http::embedded::DebugToolsPlugin;
pub use crate::debug_http::snapshot::target_status_line;
pub use crate::sim_core::SimulationDebugLog;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::debug_http::{
    poll_debug_http, start_debug_http_server, try_start_debug_http_server, DebugHttpBridge,
    PendingDebugHttpStart,
};
