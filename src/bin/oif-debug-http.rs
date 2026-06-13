use std::sync::{Arc, Mutex};

use open_infinifactory::debug_http::fixture::{apply_fixture_setup, load_fixture_file};
use open_infinifactory::debug_http::standalone::{HeadlessDebugState, run_headless_server};
use open_infinifactory::debug_http::world_ops::load_save_into_session;
use open_infinifactory::shared::launch::{LaunchOptions, DEFAULT_DEBUG_HTTP_PORT};

fn main() {
    let launch = LaunchOptions::from_args();
    let port = launch
        .debug_http_port
        .unwrap_or(DEFAULT_DEBUG_HTTP_PORT);

    let mut state = HeadlessDebugState::new();
    state.with_core(|mut core, sim_log| {
        sim_log.set_enabled(true);
        if let Some(save) = &launch.load_save {
            if let Err(error) = load_save_into_session(&mut core, save) {
                eprintln!("failed to load save `{save}`: {error}");
                std::process::exit(1);
            }
            sim_log.log(0, format!("loaded save `{save}`"));
        }
        if let Some(fixture) = &launch.load_fixture {
            match load_fixture_file(fixture).and_then(|fixture| {
                apply_fixture_setup(&mut core, &fixture)?;
                Ok(fixture.name)
            }) {
                Ok(name) => sim_log.log(0, format!("loaded fixture `{name}`")),
                Err(error) => {
                    eprintln!("failed to load fixture `{fixture}`: {error}");
                    std::process::exit(1);
                }
            }
        }
    });

    let state = Arc::new(Mutex::new(state));
    run_headless_server(state, port);
}
