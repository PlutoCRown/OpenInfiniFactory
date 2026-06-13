use std::sync::mpsc;

#[derive(Debug)]
pub enum DebugHttpCommand {
    GetPosBlock {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        world: Option<String>,
    },
    GetExtendedDevices,
    GetStatus,
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
    GetRegion {
        min_x: Option<i32>,
        min_y: Option<i32>,
        min_z: Option<i32>,
        max_x: Option<i32>,
        max_y: Option<i32>,
        max_z: Option<i32>,
        world: Option<String>,
    },
    GetPowerNetworks,
    GetPowerNetwork { id: Option<usize> },
    GetPoweredDevices,
    GetFactoryBlockState {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        world: Option<String>,
    },
    GetFactoryIdAt {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        world: Option<String>,
    },
    GetFactoryPos {
        id: Option<u32>,
        world: Option<String>,
    },
    GetStructureAt {
        x: Option<i32>,
        y: Option<i32>,
        z: Option<i32>,
        world: Option<String>,
    },
    PreviewMovementPlan,
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
            world: params.get("world").cloned(),
        },
        ("GET", "/getextendeddevices") | ("GET", "/extendeddevices") => {
            DebugHttpCommand::GetExtendedDevices
        }
        ("GET", "/status") => DebugHttpCommand::GetStatus,
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
        ("GET", "/getregion") => DebugHttpCommand::GetRegion {
            min_x: params.get("min_x").or_else(|| params.get("minx")).and_then(|v| v.parse().ok()),
            min_y: params.get("min_y").or_else(|| params.get("miny")).and_then(|v| v.parse().ok()),
            min_z: params.get("min_z").or_else(|| params.get("minz")).and_then(|v| v.parse().ok()),
            max_x: params.get("max_x").or_else(|| params.get("maxx")).and_then(|v| v.parse().ok()),
            max_y: params.get("max_y").or_else(|| params.get("maxy")).and_then(|v| v.parse().ok()),
            max_z: params.get("max_z").or_else(|| params.get("maxz")).and_then(|v| v.parse().ok()),
            world: params.get("world").cloned(),
        },
        ("GET", "/getpowernetworks") | ("GET", "/powernetworks") => {
            DebugHttpCommand::GetPowerNetworks
        }
        ("GET", "/getpowernetwork") | ("GET", "/powernetwork") => DebugHttpCommand::GetPowerNetwork {
            id: params
                .get("id")
                .or_else(|| params.get("network_id"))
                .and_then(|v| v.parse().ok()),
        },
        ("GET", "/getpowereddevices") | ("GET", "/powereddevices") => {
            DebugHttpCommand::GetPoweredDevices
        }
        ("GET", "/getfactoryblockstate") | ("GET", "/factoryblockstate") => {
            DebugHttpCommand::GetFactoryBlockState {
                x: params.get("x").and_then(|v| v.parse().ok()),
                y: params.get("y").and_then(|v| v.parse().ok()),
                z: params.get("z").and_then(|v| v.parse().ok()),
                world: params.get("world").cloned(),
            }
        }
        ("GET", "/getfactoryidat") | ("GET", "/factoryidat") => DebugHttpCommand::GetFactoryIdAt {
            x: params.get("x").and_then(|v| v.parse().ok()),
            y: params.get("y").and_then(|v| v.parse().ok()),
            z: params.get("z").and_then(|v| v.parse().ok()),
            world: params.get("world").cloned(),
        },
        ("GET", "/getfactorypos") | ("GET", "/factorypos") => DebugHttpCommand::GetFactoryPos {
            id: params.get("id").and_then(|v| v.parse().ok()),
            world: params.get("world").cloned(),
        },
        ("GET", "/getstructureat") | ("GET", "/structureat") => DebugHttpCommand::GetStructureAt {
            x: params.get("x").and_then(|v| v.parse().ok()),
            y: params.get("y").and_then(|v| v.parse().ok()),
            z: params.get("z").and_then(|v| v.parse().ok()),
            world: params.get("world").cloned(),
        },
        ("GET", "/previewmovementplan") | ("GET", "/movementplan") => {
            DebugHttpCommand::PreviewMovementPlan
        }
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
            {"method": "GET", "path": "/getPosBlock?x=&y=&z=&world=turn|solution", "desc": "block at coordinate (default world=turn)"},
            {"method": "GET", "path": "/getExtendedDevices", "desc": "extended pusher/blocker positions"},
            {"method": "GET", "path": "/status", "desc": "simulation snapshot"},
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
            {"method": "GET", "path": "/getRegion?min_x=&min_y=&min_z=&max_x=&max_y=&max_z=&world=turn|solution", "desc": "blocks in axis-aligned region (default world=turn)"},
            {"method": "GET", "path": "/getPowerNetworks", "desc": "all wire network summaries"},
            {"method": "GET", "path": "/getPowerNetwork?id=", "desc": "one wire network detail"},
            {"method": "GET", "path": "/getPoweredDevices", "desc": "currently powered pusher/blocker devices"},
            {"method": "GET", "path": "/getFactoryBlockState?x=&y=&z=&world=turn|solution", "desc": "structure + signal state for factory cell (default world=turn)"},
            {"method": "GET", "path": "/getFactoryIdAt?x=&y=&z=&world=turn|solution", "desc": "factory block id at coordinate (default world=turn)"},
            {"method": "GET", "path": "/getFactoryPos?id=&world=turn|solution", "desc": "factory block position by id (default world=turn)"},
            {"method": "GET", "path": "/getStructureAt?x=&y=&z=&world=turn|solution", "desc": "full connected structure members (default world=turn)"},
            {"method": "GET", "path": "/previewMovementPlan", "desc": "movement plan preview with execute feasibility"},
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
