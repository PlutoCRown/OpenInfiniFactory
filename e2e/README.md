# E2E 测试（模拟核心 + Debug HTTP）

通过无头 `oif-debug-http`（Bevy ECS App，无窗口）驱动 `SimCoreWorld`，对每个方块做放置与（部分）模拟断言。

## 前置

- [Bun](https://bun.sh)
- Rust 工具链（用于编译 `oif-debug-http`）

## 生成方块 fixture

```bash
cd e2e
bun run generate-fixtures
```

会在 `e2e/fixtures/blocks/` 下为每个 `BlockKind` 生成 JSON。

## 运行测试

```bash
cd e2e
bun test
```

测试会：

1. 启动 `cargo run --bin oif-debug-http -- --debug-http=9876`
2. 调用 `GET /blockKinds` 校验注册表
3. 调用 `POST /runAllFixtures` 跑全部方块放置用例
4. 调用 `POST /runFixture` 跑 `fixtures/sim/` 下的模拟用例
5. 调用 `POST /runN` 校验多回合推进

## 手动调试

```bash
cargo run --bin oif-debug-http -- --debug-http=8765
curl http://127.0.0.1:8765/status
curl -X POST 'http://127.0.0.1:8765/runFixture?path=blocks/Platform.json'
curl -X POST http://127.0.0.1:8765/runAllFixtures
```

## 目录

| 路径 | 说明 |
|------|------|
| `fixtures/blocks/` | 每方块放置 + assert |
| `fixtures/sim/` | 需要跑回合的行为用例 |
| `src/client.ts` | HTTP 客户端 |
| `src/blocks.test.ts` | 主测试入口 |
