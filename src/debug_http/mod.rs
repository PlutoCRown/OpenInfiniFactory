pub mod embedded;
pub mod embedded_session;
pub mod fixture;
pub mod headless;
pub mod introspection;
pub mod protocol;
pub mod snapshot;
pub mod world_layer;
pub mod world_ops;

#[cfg(not(target_arch = "wasm32"))]
pub mod standalone;

pub use embedded::DebugToolsPlugin;
pub use protocol::DebugHttpCommand;
pub use snapshot::{
    block_json, cursor_target_json, pos_json, simulation_status_json, target_status_line,
};

#[cfg(not(target_arch = "wasm32"))]
pub use embedded::{poll_debug_http, start_debug_http_server, DebugHttpBridge};
