---
name: sim-debug-http
description: >-
  OpenInfiniFactory 模拟调试：用 Free/Puzzle/Solution 存档 + oif-debug-http
  复现并调试 SimSession / WorldBlocks。材料不入档，需要时用 HTTP place。
---

# 模拟 HTTP 调试（OpenInfiniFactory）

## 交互式修复玩家问题时的安全规则

调试器内嵌在游戏客户端时，先读取：

```bash
curl -s http://127.0.0.1:8765/status
```

从响应的 `save.path` 获取当前加载存档路径，从 `save.dirty` 或
`save.exit_requires_save` 判断退出/重启是否会提示保存。

只有在以下条件满足时，才允许替玩家重启客户端：

1. `save.path` 非空，并且确认是玩家当前正在看的存档。
2. `save.dirty == false` 且 `save.exit_requires_save == false`。
3. 重启前都先调用 `POST /session/save`，这样即使世界本身不脏，也会把玩家当前坐标和朝向写入存档；如果存档原本是脏的，再轮询 `/status`，直到脏状态变成 false。此规则只适用于玩家已有的正式存档。
4. 保存失败、状态查询失败、存档路径不明或状态仍为脏时，禁止擅自关闭玩家客户端；应把阻塞原因告诉玩家。

保存完成后，可以用同一个路径启动并回到玩家位置：

```bash
cargo run -- --debug-http --load-save=<save.path>
```

游戏客户端启动时建议默认带 `--debug-http`，这样重启后仍能继续查询玩家、准星、方块和模拟状态。

## 玩家说“看一下”时的方块读取顺序

如果玩家看起来正处于游戏中，并且已经启用了调试器，优先读取玩家准星指向的方块：

1. 先查 `/status`，读取 `cursor.pos` 和 `cursor.block`；这里是当前准星命中的方块及其完整调试信息。
2. 如果没有读到 `cursor.block`，必须明确告诉玩家“我没读到指向的方块信息”，不能把附近方块冒充成准星目标。
3. 在说明上述限制后，再用 `/players` 获取玩家位置，并用 `/blocks` 查询附近方块；如果玩家讨论的是明确类型，只筛选该类型。
4. 如果玩家讨论的是活塞等普通方块类型，使用 `kind=Pusher` 等精确筛选，并设置合理的 `radius` 和 `limit`；验收器属于特殊结构，继续使用专用的 `/acceptors`，不要假设存在 `kind=Acceptor`。

## 创建独立测试用例

`--create-test-free[=NAME]` 会立即创建并进入一个仅存在于当前进程内存的 Free 测试存档。存档默认名为
`test_free`，若重名会自动追加后缀；初始世界只在 `(0, 0, 0)` 放置一个草地方块，
并带有默认玩家位置，适合验证渲染和输入问题。它不会进入持久化队列，测试进程结束后用例自动消失：

```bash
cargo run -- --create-test-free=android_render_case --debug-http
```

通过 `/status` 确认已进入测试世界，然后按需放置方块、执行模拟。测试用例禁止调用
`POST /session/save`，也不要尝试通过 `--load-save` 在新进程中恢复它：

```bash
curl -X POST 'http://127.0.0.1:8765/world/place?x=1&y=0&z=0&kind=grass'
curl -X POST 'http://127.0.0.1:8765/sim/begin'
curl -X POST 'http://127.0.0.1:8765/sim/run?n=10'
```

需要展示给玩家时，直接把当前这个带 `--debug-http` 的游戏客户端交给玩家查看；如果测试进程
结束，应重新创建测试用例，不要把它转成正式存档。无头调试器仍可创建、放置和执行模拟，但
`/session/save` 对无头测试明确禁用。

完整 API 见 [http-api.md](http-api.md)。

## 启动

```bash
cargo run --bin oif-debug-http -- --debug-http=8765 --load-save=free_sandbox
# 创建单方块 Free 测试存档并进入
cargo run --bin oif-debug-http -- --debug-http=8765 --create-test-free=test_case
# 或游戏内嵌
cargo run -- --debug-http --load-save=free_sandbox
```

无头优先支持完整 session / sim / world API；内嵌侧重查询、玩家、传送与 status/perf。

## 相关源码

| 路径 | 说明 |
|------|------|
| `src/bin/oif-debug-http.rs` | 无头 HTTP 入口 |
| `src/debug_http/protocol.rs` | 路由 |
| `src/debug_http/headless.rs` | 无头处理 |
| `src/debug_http/embedded.rs` | 游戏内嵌处理 |
| `src/debug_http/snapshot.rs` | JSON 快照辅助 |
| `crates/oif-sim/.../signals.rs` | `SignalNetworkCache::query_power_at` |
