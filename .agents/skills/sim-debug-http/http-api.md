# Debug HTTP API 参考

服务：`cargo run --bin oif-debug-http -- --debug-http=<PORT>`

基址：`http://127.0.0.1:<PORT>`

实现：`src/debug_http/protocol.rs`（路由）、`src/debug_http/headless.rs`（headless 处理）。嵌入游戏模式同样协议，由 `src/debug_http/embedded.rs` 处理。

---

## 查询

### GET `/help` 或 GET `/`

返回 `{ ok, endpoints: [{ method, path, desc }] }`。

### GET `/getPosBlock?x=&y=&z=`

别名：`/getBlock`

返回：

```json
{
  "ok": true,
  "pos": { "x": 0, "y": 1, "z": 0 },
  "block": { "kind": "Platform", "facing": "North", "layer": "factory" }
}
```

无方块时 `block` 为 `null`。headless 模式缺少 x/y/z 会 `{ ok: false, error: "..." }`。

### GET `/status`

```json
{
  "ok": true,
  "simulation": {
    "active": false,
    "mode": "headless",
    "running": false,
    "speed": 1,
    "step_requested": false,
    "turn": 0
  }
}
```

### GET `/blockKinds`

别名：`/blocks`

返回全部 `BlockKind` 及 `layer`（scene/factory/system/material/virtual）、`directional`。

### GET `/logs?limit=100`

返回 `{ ok, logs: string[] }`。日志含 `[sim turn=N]` 前缀的结构化模拟输出。

---

## 世界与会话

### POST `/world/reset`

清空当前会话中的方块与模拟状态。

### POST `/world/place?x=&y=&z=&kind=&facing=`

放置单个方块。`kind` 大小写不敏感（如 `pusher`）；`facing`: `North`/`East`/`South`/`West` 或 `N`/`E`/`S`/`W`。

失败示例：`cannot place Pusher at (0, 0, 0)`（占格规则不通过）。

### POST `/loadSave?name=<save_stem>`

加载 `saves/<name>.ron`。Solution 会自动加载其 `puzzle_id` 对应 Puzzle 并合并工厂层。

---

## Fixture

路径参数 `path` 相对 **`e2e/fixtures/`**（非 URL 编码）。

### POST `/loadFixture?path=sim/foo.json`

仅执行 fixture 的 `setup`（放置方块），不跑 `steps`。

### POST `/runFixture?path=sim/foo.json`

`setup` + 逐步执行 `steps`。某步失败返回 `{ ok: false, error: "case step 2: ..." }`。

### POST `/runAllFixtures`

运行 `e2e/fixtures/blocks/` 下全部 `*.json`（单方块放置 smoke），返回 `{ passed, total, results }`。

---

## 模拟推进

| 端点 | 行为 |
|------|------|
| POST `/beginSimulation` | 拍快照，允许后续回合 |
| POST `/runOneTurn` | 先 beginSimulation，再 1 回合 |
| POST `/runN?n=` | 先 beginSimulation，再 n 回合（n≥1） |
| POST `/run` | headless：连续 10 回合；响应含 `note` |

每回合时长：`SIMULATION_TURN_SECONDS`（见 `src/game/world/animation.rs`）。

---

## 日志

### DELETE `/logs` 或 POST `/clearLogs`

清空模拟日志缓冲区。

---

## 启动参数（`oif-debug-http`）

| 参数 | 说明 |
|------|------|
| `--debug-http=<PORT>` | 监听端口 |
| `--load-save=<name>` | 启动时加载存档 |
| `--load-fixture=<path>` | 启动时加载 fixture setup |

---

## Fixture JSON schema

```typescript
{
  name: string;
  source?: { save: string; min: [number,number,number]; max: [...]; origin: [...] }; // 仅 export_fixture 写入
  setup: Array<{ x: number; y: number; z: number; kind: string; facing?: string }>;
  steps?: Array<
    | { op: "beginSimulation" }
    | { op: "run"; turns?: number }
    | { op: "assertBlock"; x: number; y: number; z: number; kind: string; layer?: string }
  >;
}
```

`run` step 的 `turns` 默认 1。`assertBlock` 的 `layer` 可选，用于区分 factory/scene 等同 kind 不同层。

---

## export_fixture CLI

```bash
cargo run --bin export_fixture -- \
  --solution <name> \
  --min x,y,z --max x,y,z \
  --out e2e/fixtures/sim/<file>.json \
  [--name <fixture_name>] \
  [--no-normalize] \
  [--with-run-steps] [--turns N]
```

从已加载世界（Solution→合并 Puzzle）裁剪轴对齐包围盒内所有 `blocks` + `system_blocks`，写出 fixture。
