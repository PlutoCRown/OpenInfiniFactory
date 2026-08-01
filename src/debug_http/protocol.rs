use std::sync::mpsc;

/// Debug HTTP 命令枚举
#[derive(Debug)]
pub enum DebugHttpCommand {
    GetPosBlock {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        block_id: Option<u64>,
    },
    GetStructure {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        block_id: Option<u64>,
        structure_id: Option<u64>,
    },
    GetPower {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        block_id: Option<u64>,
    },
    GetPlayers,
    GetAcceptors,
    GetStatus,
    GetPerf,
    Run,
    RunOneTurn,
    RunN {
        n: u64,
    },
    SimPause,
    GetLogs {
        limit: usize,
    },
    ClearLogs,
    Help,
    BlockKinds,
    WorldReset,
    BeginSimulation,
    LoadSave {
        name: String,
    },
    SessionExit,
    SessionSave,
    PlaceBlock {
        x: i32,
        y: i32,
        z: i32,
        kind: String,
        facing: String,
    },
    TeleportPlayer {
        x: f32,
        y: f32,
        z: f32,
        yaw: Option<f32>,
        pitch: Option<f32>,
        look_at: Option<(f32, f32, f32)>,
    },
}

/// HTTP 请求：命令 + 回传通道
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
        ("GET", "/getposblock") | ("GET", "/getblock") | ("GET", "/block") => {
            DebugHttpCommand::GetPosBlock {
                x: params.get("x").and_then(|v| v.parse().ok()),
                y: params.get("y").and_then(|v| v.parse().ok()),
                z: params.get("z").and_then(|v| v.parse().ok()),
                block_id: params.get("id").and_then(|v| v.parse().ok()),
            }
        }
        ("GET", "/getstructure") | ("GET", "/structure") => DebugHttpCommand::GetStructure {
            x: params.get("x").and_then(|v| v.parse().ok()),
            y: params.get("y").and_then(|v| v.parse().ok()),
            z: params.get("z").and_then(|v| v.parse().ok()),
            block_id: params
                .get("blockid")
                .or_else(|| params.get("block_id"))
                .and_then(|v| v.parse().ok()),
            structure_id: params
                .get("id")
                .or_else(|| params.get("structureid"))
                .or_else(|| params.get("structure_id"))
                .and_then(|v| v.parse().ok()),
        },
        ("GET", "/power") => DebugHttpCommand::GetPower {
            x: params.get("x").and_then(|v| v.parse().ok()),
            y: params.get("y").and_then(|v| v.parse().ok()),
            z: params.get("z").and_then(|v| v.parse().ok()),
            block_id: params.get("id").and_then(|v| v.parse().ok()),
        },
        ("GET", "/players") => DebugHttpCommand::GetPlayers,
        ("GET", "/acceptors") => DebugHttpCommand::GetAcceptors,
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
        ("POST", "/beginsimulation")
        | ("GET", "/beginsimulation")
        | ("POST", "/sim/begin")
        | ("GET", "/sim/begin") => DebugHttpCommand::BeginSimulation,
        ("POST", "/sim/pause") | ("GET", "/sim/pause") => DebugHttpCommand::SimPause,
        ("POST", "/loadsave")
        | ("GET", "/loadsave")
        | ("POST", "/session/enter")
        | ("GET", "/session/enter") => DebugHttpCommand::LoadSave {
            name: params.get("name").cloned().unwrap_or_default(),
        },
        ("POST", "/session/exit") | ("GET", "/session/exit") => DebugHttpCommand::SessionExit,
        ("POST", "/session/save") | ("GET", "/session/save") => DebugHttpCommand::SessionSave,
        ("POST", "/world/place") | ("GET", "/world/place") => DebugHttpCommand::PlaceBlock {
            x: params.get("x").and_then(|v| v.parse().ok()).unwrap_or(0),
            y: params.get("y").and_then(|v| v.parse().ok()).unwrap_or(0),
            z: params.get("z").and_then(|v| v.parse().ok()).unwrap_or(0),
            kind: params.get("kind").cloned().unwrap_or_default(),
            facing: params
                .get("facing")
                .cloned()
                .unwrap_or_else(|| "North".into()),
        },
        ("POST", "/run") | ("GET", "/run") => DebugHttpCommand::Run,
        ("POST", "/runoneturn") | ("GET", "/runoneturn") => DebugHttpCommand::RunOneTurn,
        ("POST", "/runn")
        | ("GET", "/runn")
        | ("POST", "/sim/run")
        | ("GET", "/sim/run") => DebugHttpCommand::RunN {
            n: params
                .get("n")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1)
                .max(1),
        },
        ("POST", "/players/0/teleport") | ("GET", "/players/0/teleport") => {
            DebugHttpCommand::TeleportPlayer {
                x: params.get("x").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                y: params.get("y").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                z: params.get("z").and_then(|v| v.parse().ok()).unwrap_or(0.0),
                yaw: params.get("yaw").and_then(|v| v.parse().ok()),
                pitch: params.get("pitch").and_then(|v| v.parse().ok()),
                look_at: params.get("lookat").and_then(|v| parse_look_at(v)),
            }
        }
        ("DELETE", "/logs") | ("POST", "/clearlogs") => DebugHttpCommand::ClearLogs,
        _ => DebugHttpCommand::Help,
    }
}

fn parse_look_at(value: &str) -> Option<(f32, f32, f32)> {
    let mut parts = value.split(',');
    let x = parts.next()?.trim().parse().ok()?;
    let y = parts.next()?.trim().parse().ok()?;
    let z = parts.next()?.trim().parse().ok()?;
    Some((x, y, z))
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            Some((
                parts.next()?.to_ascii_lowercase(),
                parts.next().unwrap_or("").to_string(),
            ))
        })
        .collect()
}

/// 端点帮助 JSON
pub fn help_json() -> String {
    serde_json::json!({
        "ok": true,
        "endpoints": [
            {"method": "GET", "path": "/block?x=&y=&z=|/block?id=", "desc": "block at coordinate or by id (alias /getPosBlock)"},
            {"method": "GET", "path": "/structure?id=|/blockId=|x=&y=&z=", "desc": "structure by id / block id / position"},
            {"method": "GET", "path": "/power?x=&y=&z=|/power?id=", "desc": "signal network (wires + devices) at position/block id"},
            {"method": "GET", "path": "/players", "desc": "player camera position + look target"},
            {"method": "GET", "path": "/acceptors", "desc": "acceptor structures with accepted counts"},
            {"method": "GET", "path": "/status", "desc": "session + simulation snapshot"},
            {"method": "GET", "path": "/perf", "desc": "load_ms + sim turn + frame timing"},
            {"method": "GET", "path": "/blockKinds", "desc": "all registered block kinds"},
            {"method": "POST", "path": "/session/enter?name=", "desc": "load save (alias /loadSave)"},
            {"method": "POST", "path": "/session/exit", "desc": "exit world / reset headless session"},
            {"method": "POST", "path": "/session/save", "desc": "save current world (embedded only)"},
            {"method": "POST", "path": "/world/reset", "desc": "clear session world"},
            {"method": "POST", "path": "/world/place?x=&y=&z=&kind=&facing=", "desc": "place one block"},
            {"method": "POST", "path": "/sim/begin", "desc": "begin simulation (alias /beginSimulation)"},
            {"method": "POST", "path": "/sim/pause", "desc": "stop continuous run"},
            {"method": "POST", "path": "/sim/run?n=", "desc": "advance N turns (alias /runN)"},
            {"method": "POST", "path": "/runOneTurn", "desc": "advance one turn"},
            {"method": "POST", "path": "/players/0/teleport?x=&y=&z=&yaw=&pitch=&lookAt=", "desc": "teleport player (embedded)"},
            {"method": "GET", "path": "/logs?limit=100", "desc": "recent simulation logs"},
            {"method": "DELETE", "path": "/logs", "desc": "clear logs"},
        ]
    })
    .to_string()
}

/// 错误响应
pub fn json_error(message: &str) -> String {
    serde_json::json!({ "ok": false, "error": message }).to_string()
}

/// 成功响应（合并 ok:true）
pub fn json_ok(data: serde_json::Value) -> String {
    let mut value = data;
    if let Some(object) = value.as_object_mut() {
        object.insert("ok".into(), true.into());
    }
    value.to_string()
}
