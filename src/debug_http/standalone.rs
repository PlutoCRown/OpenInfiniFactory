use bevy::prelude::*;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::sim_core::{build_headless_sim_app, SimCoreWorld, SimulationDebugLog};

use super::headless::handle_headless_command;
use super::protocol::{parse_http_request, DebugHttpRequest};

pub struct HeadlessDebugState {
    pub app: App,
}

impl HeadlessDebugState {
    pub fn new() -> Self {
        Self {
            app: build_headless_sim_app(),
        }
    }

    pub fn with_core<R>(
        &mut self,
        f: impl FnOnce(SimCoreWorld<'_>, &mut SimulationDebugLog) -> R,
    ) -> R {
        self.app.world_mut().resource_scope(
            |world, mut sim_log: Mut<SimulationDebugLog>| {
                let core = SimCoreWorld::new(world);
                f(core, &mut sim_log)
            },
        )
    }
}

pub fn run_headless_server(state: Arc<Mutex<HeadlessDebugState>>, port: u16) {
    let (request_tx, request_rx) = mpsc::channel();
    let listen_addr = format!("127.0.0.1:{port}");
    let thread_tx = request_tx.clone();

    let listener = thread::spawn(move || run_http_thread(&listen_addr, thread_tx));

    eprintln!("OpenInfiniFactory headless debug HTTP: http://127.0.0.1:{port}");

    for request in request_rx {
        let body = {
            let mut state = state.lock().expect("headless debug state lock");
            handle_headless_command(&mut state, request.command)
        };
        let _ = request.respond_to.send(body);
    }

    let _ = listener.join();
}

pub fn run_http_thread(listen_addr: &str, request_tx: mpsc::Sender<DebugHttpRequest>) {
    let server = match tiny_http::Server::http(listen_addr) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("debug HTTP failed to bind {listen_addr}: {error}");
            return;
        }
    };

    for request in server.incoming_requests() {
        let (response_tx, response_rx) = mpsc::channel();
        let command = parse_http_request(&request);
        if request_tx
            .send(DebugHttpRequest {
                command,
                respond_to: response_tx,
            })
            .is_err()
        {
            break;
        }

        let (status, body) = match response_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(body) => (200, body),
            Err(_) => (
                504,
                r#"{"ok":false,"error":"handler timeout"}"#.into(),
            ),
        };
        let response = tiny_http::Response::from_string(body)
            .with_status_code(status)
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .expect("valid header"),
            )
            .with_header(
                tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..])
                    .expect("valid header"),
            );
        let _ = request.respond(response);
    }
}
