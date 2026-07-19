---
name: sim-debug-http
description: >-
  OpenInfiniFactory 模拟调试：从存档区域或文字描述的方块结构导出 e2e fixture，
  启动 oif-debug-http 无头服务，用 HTTP 复现并调试 SimCoreWorld 逻辑。
  在用户描述存档某坐标范围的行为异常、某方块没被推动/没通电、
  或直接文字描述方块布局要查模拟问题时使用。
---

# 模拟 HTTP 调试（OpenInfiniFactory）

## 何时启用

- 玩家说某个 **存档** 在某个 **坐标范围** 行为不对
- 玩家 **文字描述方块结构**（种类、朝向、相对位置）要查模拟逻辑
- 需要最小化复现、写回归用例、或定位 `SimCoreWorld` / 结构移动 / 信号等问题

## 总流程（两步，不可跳过验证）

```
1. 导出/制作 fixture → 验证与原始描述一致
2. 启动 HTTP 服务 → 加载 fixture → 逐回合模拟 → 读 logs / getPosBlock 定位原因
```

**第一步未验证前，不要开始改游戏逻辑。**

---

## 第一步：导出或制作测试用例

### 路径 A：从存档裁剪（优先）

存档在 `saves/*.ron`。**Puzzle** 只有场景/系统方块；**Solution** 只有 `puzzle_id` + 工厂方块，加载时会自动合并对应 Puzzle。

```bash
cargo run --bin export_fixture -- \
  --solution <solution_name> \
  --min x,y,z --max x,y,z \
  --out e2e/fixtures/sim/<case_name>.json \
  --name <case_name> \
  --with-run-steps --turns <N>
```

- `--min` / `--max`：对角两点即可，工具会自动归一化边界
- 默认把 `--min` 角移到本地 `(0,0,0)`；保留世界坐标用 `--no-normalize`
- `source` 字段会写入原始 save、范围、origin，便于对照
- **工厂方块在 Solution 里**：若用户只提 Puzzle 名，先找关联 Solution（`saves/` 里 `puzzle_id`）

**验证导出**（必须做其一）：

```bash
# 对比存档 vs fixture 方块
bun e2e/scripts/debug-pusher-fixture.ts   # 可参考改路径/坐标

# 或手动
cargo run --bin oif-debug-http -- --debug-http=8765
curl -X POST 'http://127.0.0.1:8765/loadSave?name=<solution>'
curl 'http://127.0.0.1:8765/getPosBlock?x=&y=&z='
curl -X POST 'http://127.0.0.1:8765/world/reset'
curl -X POST 'http://127.0.0.1:8765/loadFixture?path=sim/<case>.json'
curl 'http://127.0.0.1:8765/getPosBlock?x=&y=&z='   # 用本地坐标
```

### 路径 B：玩家文字描述 → 手写 fixture

写入 `e2e/fixtures/sim/<case>.json`：

```json
{
  "name": "case_name",
  "setup": [
    { "x": 0, "y": 0, "z": 0, "kind": "Stone", "facing": "North" },
    { "x": 0, "y": 1, "z": 0, "kind": "Pusher", "facing": "East" }
  ],
  "steps": [
    { "op": "beginSimulation" },
    { "op": "run", "turns": 2 },
    { "op": "assertBlock", "x": 1, "y": 1, "z": 0, "kind": "Platform", "layer": "factory" }
  ]
}
```

**放置顺序**：scene → factory → system → material → virtual（`export_fixture` 已按此排序）。

**Step 类型**（`src/debug_http/fixture.rs`）：

| op | 字段 | 说明 |
|----|------|------|
| `beginSimulation` | — | 拍快照，进入模拟 |
| `run` | `turns` | 跑 N 回合（内部会先 beginSimulation） |
| `assertBlock` | `x,y,z,kind`, 可选 `layer` | 断言方块；layer: `scene`/`factory`/`system`/`material`/`virtual` |

`kind` / `facing` 与 Rust `BlockKind` / `Facing` 的 Debug 名一致（如 `Pusher`, `East`）。

---

## 第二步：HTTP 调试

### 启动服务

```bash
cargo run --bin oif-debug-http -- --debug-http=8765
# 可选：直接加载存档或 fixture
cargo run --bin oif-debug-http -- --debug-http=8765 --load-save=solution_3
cargo run --bin oif-debug-http -- --load-fixture=sim/test_pusher_platform_stuck.json
```

默认端口见 `DEFAULT_DEBUG_HTTP_PORT`（通常 8765）。e2e 测试用 9876。

### 推荐调试序列

```bash
curl -X POST 'http://127.0.0.1:8765/world/reset'
curl -X POST 'http://127.0.0.1:8765/loadFixture?path=sim/<case>.json'
curl -X POST 'http://127.0.0.1:8765/beginSimulation'
curl -X POST 'http://127.0.0.1:8765/runOneTurn'    # 逐回合，便于读 log
curl 'http://127.0.0.1:8765/logs?limit=80'
curl 'http://127.0.0.1:8765/getPosBlock?x=3&y=2&z=1'
curl -X POST 'http://127.0.0.1:8765/runN?n=2'        # 或一次跑多回合
curl -X POST 'http://127.0.0.1:8765/runFixture?path=sim/<case>.json'  # 含 steps 断言
```

或用 Bun 脚本（可拷贝修改）：

- `e2e/scripts/debug-pusher-fixture.ts` — 存档 vs fixture 对比 + 跑 N 回合
- `e2e/scripts/debug-pusher-turns.ts` — 逐回合打印 signals/gravity/devices/merged
- `e2e/src/client.ts` — `spawnDebugServer` / `DebugClient`

### 读日志

`/logs` 里关注：

- `signals` — 通电网络与 powered device
- `actuating pusher/blocker` — 活塞/阻拦器动作
- `gravity` / `devices` / `merged` — 移动计划（水平 Push 在 `devices`）
- `translate ... mark=Push` — 结构被推动

`getPosBlock` 返回完整方块信息：`kind`/`facing`/`yaw`/`paints`/`attached_stamps`/`settings` 等，或空。

---

## HTTP 端点一览

完整说明见 [http-api.md](http-api.md)。GET/POST 均可（query 传参）。

| 方法 | 路径 | 作用 |
|------|------|------|
| GET | `/help` | 端点列表 JSON |
| GET | `/getPosBlock?x=&y=&z=` | 查询坐标方块（headless 必填 xyz） |
| GET | `/status` | 模拟状态（turn、running 等） |
| GET | `/blockKinds` | 全部已注册 BlockKind + layer |
| POST | `/world/reset` | 清空会话世界 |
| POST | `/world/place?x=&y=&z=&kind=&facing=` | 放置单方块 |
| POST | `/loadSave?name=` | 按存档名加载（合并 puzzle+solution） |
| POST | `/loadFixture?path=` | 只应用 fixture 的 `setup` |
| POST | `/runFixture?path=` | 加载并执行 `setup` + `steps` |
| POST | `/runAllFixtures` | 跑 `e2e/fixtures/blocks/*.json` |
| POST | `/beginSimulation` | 进入模拟快照 |
| POST | `/runOneTurn` | 推进 1 回合（会先 beginSimulation） |
| POST | `/runN?n=` | 推进 N 回合 |
| POST | `/run` | headless 下立即跑 10 回合 |
| GET | `/logs?limit=` | 最近模拟日志（默认 100） |
| DELETE | `/logs` | 清空日志 |

响应格式：`{ "ok": true, ... }` 或 `{ "ok": false, "error": "..." }`。

Fixture 路径相对 `e2e/fixtures/`（如 `sim/foo.json`、`blocks/Platform.json`）。**不要用 URL 编码的 `/`**（`sim%2Ffoo` 会失败）。

---

## 修复后

1. 在 fixture 的 `steps` 里加 `assertBlock` 作回归
2. 复杂结构可加 Rust 单元测试（`structure_state` 等）
3. 跑 `curl -X POST .../runFixture?path=sim/<case>.json` 或 `cd e2e && bun test`
4. 较大改动后：`bun scripts/log_rs_lines.js` 报告行数

---

## 相关源码

| 路径 | 说明 |
|------|------|
| `src/bin/oif-debug-http.rs` | 无头 HTTP 入口 |
| `src/bin/export_fixture.rs` | 存档区域 → fixture |
| `src/debug_http/protocol.rs` | 路由与 help_json |
| `src/debug_http/fixture.rs` | fixture 格式与执行 |
| `src/debug_http/headless.rs` | 命令处理 |
| `e2e/fixtures/sim/` | 行为/回归用例 |
| `e2e/README.md` | e2e 总览 |

## 注意

- 模拟逻辑在 ECS / `SimCoreWorld`；UI 问题不走此 skill
- 项目未发布，旧存档格式用 `export_fixture` 或手动改 `saves/`，不在游戏内做 legacy 兼容
- 正常开发无需 `cargo check`/`cargo run` 游戏本体，但 **HTTP 调试必须能编译 `oif-debug-http`**
