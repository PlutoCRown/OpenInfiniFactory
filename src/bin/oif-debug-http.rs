use open_infinifactory::debug_http::standalone::{run_headless_server, HeadlessDebugState};
use open_infinifactory::debug_http::world_ops::load_save_into_session;
use open_infinifactory::shared::launch::{LaunchOptions, DEFAULT_DEBUG_HTTP_PORT};
use std::sync::{Arc, Mutex};

fn main() {
    let launch = LaunchOptions::from_args();
    let port = launch
        .debug_http_port
        .unwrap_or(DEFAULT_DEBUG_HTTP_PORT);

    let mut state = HeadlessDebugState::new();
    state.with_core(|core| {
        core.log.set_enabled(true);
    });
    if let Some(save) = &launch.load_save {
        match state.with_core(|core| load_save_into_session(core, save)) {
            Ok(load_ms) => {
                state.current_save = Some(save.clone());
                state.last_load_ms = Some(load_ms);
                state
                    .session
                    .log
                    .log(0, format!("loaded save `{save}` in {load_ms:.1}ms"));
            }
            Err(error) => {
                eprintln!("failed to load save `{save}`: {error}");
                std::process::exit(1);
            }
        }
    }

    let state = Arc::new(Mutex::new(state));
    run_headless_server(state, port);
}
