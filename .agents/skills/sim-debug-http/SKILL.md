---
name: sim-debug-http
description: >-
  OpenInfiniFactory 模拟调试：用 Free/Puzzle/Solution 存档 + oif-debug-http
  复现并调试 SimSession / WorldBlocks。材料不入档，需要时用 HTTP place。
---

# 模拟 HTTP 调试（OpenInfiniFactory）

## 流程

1. 人工创建 **Free**（或 Puzzle/Solution）存档
2. `cargo run --bin oif-debug-http -- --load-save=<name> --debug-http=8765`
3. HTTP：可选 `POST /world/place` 放材料 → `POST /sim/begin` → `POST /sim/run?n=` → 查 `/block` `/power` `/acceptors` `/logs`

完整 API 见 [http-api.md](http-api.md)。

## 启动

```bash
cargo run --bin oif-debug-http -- --debug-http=8765 --load-save=free_sandbox
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
