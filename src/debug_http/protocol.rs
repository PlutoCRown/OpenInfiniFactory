use std::sync::mpsc;

#[derive(Debug)]
pub enum DebugHttpCommand {
    GetPosBlock {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
    },
    GetStatus,
    GetPerf,
    Run,
    RunOneTurn,
    RunN { n: u64 },
    GetLogs { limit: usize },
    ClearLogs,
    Help,
    BlockKinds,
    WorldReset,
    BeginSimulation,
    LoadSave { name: String },
    PlaceBlock {
        x: i32,
        y: i32,
        z: i32,
        kind: String,
        facing: String,
    },
    LoadFixture { path: String },
    RunFixture { path: String },
    RunAllFixtures,
}

pub struct DebugHttpRequest {
    pub command: DebugHttpCommand,
    pub respond_to: mpsc::Sender<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_http_request(request: &tiny_http::Request) -> DebugHttpCommand {
    let method = request.method().as_str().to_ascii_uppercase();
    let path = request.url().split('?').next().unwrap_or("/");
    let path = path.trim_end_matches('/').to_ascii_lowercase();
    let path = if path.is_empty() { "/".into() } else { path };

    let query = request.url().split('?').nth(1).unwrap_or_default();
    let params = parse_query(query);

    match (method.as_str(), path.as_str()) {
        ("GET", "/") | ("GET", "/help") => DebugHttpCommand::Help,
        ("GET", "/getposblock") | ("GET", "/getblock") => DebugHttpCommand::GetPosBlock {
            x: params.get("x").and_then(|v| v.parse().ok()),
            y: params.get("y").and_then(|v| v.parse().ok()),
            z: params.get("z").and_then(|v| v.parse().ok()),
        },
        ("GET", "/status") => DebugHttpCommand::GetStatus,
        ("GET", "/perf") => DebugHttpCommand::GetPerf,
        ("GET", "/blockkinds") | ("GET", "/blocks") => DebugHttpCommand::BlockKinds,
        ("GET", "/logs") => DebugHttpCommand::GetLogs {
            limit: params
                .get("limit")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        },
        ("POST", "/world/reset") | ("GET", "/world/reset") => DebugHttpCommand::WorldReset,
        ("POST", "/beginsimulation") | ("GET", "/beginsimulation") => {
            DebugHttpCommand::BeginSimulation
        }
        ("POST", "/loadsave") | ("GET", "/loadsave") => DebugHttpCommand::LoadSave {
            name: params
                .get("name")
                .cloned()
                .unwrap_or_default(),
        },
        ("POST", "/world/place") | ("GET", "/world/place") => DebugHttpCommand::PlaceBlock {
            x: params.get("x").and_then(|v| v.parse().ok()).unwrap_or(0),
            y: params.get("y").and_then(|v| v.parse().ok()).unwrap_or(0),
            z: params.get("z").and_then(|v| v.parse().ok()).unwrap_or(0),
            kind: params.get("kind").cloned().unwrap_or_default(),
            facing: params.get("facing").cloned().unwrap_or_else(|| "North".into()),
        },
        ("POST", "/loadfixture") | ("GET", "/loadfixture") => DebugHttpCommand::LoadFixture {
            path: params.get("path").cloned().unwrap_or_default(),
        },
        ("POST", "/runfixture") | ("GET", "/runfixture") => DebugHttpCommand::RunFixture {
            path: params.get("path").cloned().unwrap_or_default(),
        },
        ("POST", "/runallfixtures") | ("GET", "/runallfixtures") => DebugHttpCommand::RunAllFixtures,
        ("POST", "/run") | ("GET", "/run") => DebugHttpCommand::Run,
        ("POST", "/runoneturn") | ("GET", "/runoneturn") => DebugHttpCommand::RunOneTurn,
        ("POST", "/runn") | ("GET", "/runn") => DebugHttpCommand::RunN {
            n: params
                .get("n")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1)
                .max(1),
        },
        ("DELETE", "/logs") | ("POST", "/clearlogs") => DebugHttpCommand::ClearLogs,
        _ => DebugHttpCommand::Help,
    }
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            Some((parts.next()?.to_string(), parts.next().unwrap_or("").to_string()))
        })
        .collect()
}

pub fn help_json() -> String {
    serde_json::json!({
        "ok": true,
        "endpoints": [
            {"method": "GET", "path": "/getPosBlock?x=&y=&z=", "desc": "block at coordinate"},
            {"method": "GET", "path": "/status", "desc": "session + simulation snapshot (any game mode)"},
            {"method": "GET", "path": "/perf", "desc": "frame timing stats (in-game overlay)"},
            {"method": "GET", "path": "/blockKinds", "desc": "all registered block kinds"},
            {"method": "POST", "path": "/world/reset", "desc": "clear session world"},
            {"method": "POST", "path": "/world/place?x=&y=&z=&kind=&facing=", "desc": "place one block"},
            {"method": "POST", "path": "/loadSave?name=", "desc": "load game save by name"},
            {"method": "POST", "path": "/loadFixture?path=", "desc": "apply fixture setup only"},
            {"method": "POST", "path": "/runFixture?path=", "desc": "load fixture and run steps"},
            {"method": "POST", "path": "/runAllFixtures", "desc": "run every e2e/fixtures/blocks/*.json"},
            {"method": "POST", "path": "/beginSimulation", "desc": "snapshot world for simulation"},
            {"method": "POST", "path": "/runOneTurn", "desc": "advance one turn"},
            {"method": "POST", "path": "/runN?n=", "desc": "advance N turns"},
            {"method": "GET", "path": "/logs?limit=100", "desc": "recent simulation logs"},
            {"method": "DELETE", "path": "/logs", "desc": "clear logs"},
        ]
    })
    .to_string()
}

pub fn json_error(message: &str) -> String {
    serde_json::json!({ "ok": false, "error": message }).to_string()
}

pub fn json_ok(data: serde_json::Value) -> String {
    let mut value = data;
    if let Some(object) = value.as_object_mut() {
        object.insert("ok".into(), true.into());
    }
    value.to_string()
}
