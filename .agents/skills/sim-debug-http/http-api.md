# Debug HTTP API 参考

服务：`cargo run --bin oif-debug-http -- --debug-http=<PORT>`

基址：`http://127.0.0.1:<PORT>`

实现：`src/debug_http/protocol.rs`、`headless.rs`、`embedded.rs`、`snapshot.rs`。

响应：`{ "ok": true, ... }` 或 `{ "ok": false, "error": "..." }`。

## Query

| 方法 | 路径 | 作用 |
|------|------|------|
| GET | `/help` | 端点列表 |
| GET | `/block?x=&y=&z=` 或 `?id=` | 方块详情（含 acceptor_count）；别名 `/getPosBlock` |
| GET | `/structure?...` | 结构查询（`id` / `blockId` / `x,y,z`）；别名 `/getStructure` |
| GET | `/power?x=&y=&z=` 或 `?id=` | 信号网络：连通分量导线 + 用电器（刷新 SignalNetworkCache） |
| GET | `/players` | `[{position, look_target}]`；无头 `[]`；内嵌为相机位置 + 准星 |
| GET | `/acceptors` | `[{id, positions, count}]`；有 StructureState 用其 count，否则世界结构 count=0 |
| GET | `/status` | `mode` / `in_world` / save / builder / entry / sim phase·turn |
| GET | `/perf` | `load_ms` + `sim_turn` + `frame`（无头 frame=null） |
| GET | `/logs?limit=` | 模拟日志 |
| GET | `/blockKinds` | 方块种类表 |

## Control

| 方法 | 路径 | 作用 |
|------|------|------|
| POST | `/session/enter?name=` | 加载存档（别名 `/loadSave`）；无头记录 current_save + load_ms |
| POST | `/session/exit` | 无头：重置世界并清空存档名；内嵌：退回主菜单 |
| POST | `/session/save` | 内嵌：保存当前世界；无头：错误 |
| POST | `/world/place?x=&y=&z=&kind=&facing=` | 放置方块（含材料） |
| POST | `/world/reset` | 清空世界（无头） |
| POST | `/sim/begin` | 进入模拟（别名 `/beginSimulation`） |
| POST | `/sim/pause` | 停止连续跑 |
| POST | `/sim/run?n=` | 推进 N 回合（别名 `/runN`；无头完整支持） |
| POST | `/runOneTurn` | 推进一回合 |
| POST | `/players/0/teleport?x&y&z&yaw&pitch` | 传送玩家（仅内嵌）；可选 `lookAt=x,y,z`（格子坐标，看向中心） |

## 启动参数

| 参数 | 说明 |
|------|------|
| `--debug-http=<PORT>` | 监听端口 |
| `--load-save=<name>` | 启动时加载存档（Free / Puzzle / Solution） |

## 备注

- 材料一般不进存档；调试时用 `/world/place` 临时放置。
- 内嵌 `session/enter`：仅主菜单可排队 `LoadWorld`；已在世界中会报错。
- 内嵌 `/sim/run?n=` 请改用无头二进制，或用 `/run` / `/runOneTurn`。
