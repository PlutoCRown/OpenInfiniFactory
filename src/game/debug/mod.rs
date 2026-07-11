//! 调试入口 re-export 与模拟日志 Resource 包装

use bevy::prelude::*;

pub use crate::debug_http::embedded::DebugToolsPlugin;
pub use crate::debug_http::snapshot::target_status_line;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::debug_http::{
    poll_debug_http, start_debug_http_server, try_start_debug_http_server, DebugHttpBridge,
    PendingDebugHttpStart,
};

/// 模拟调试日志（Bevy Resource）
#[derive(Resource, Deref, DerefMut, Default)]
pub struct SimulationDebugLog(pub oif_sim::SimulationDebugLog);
