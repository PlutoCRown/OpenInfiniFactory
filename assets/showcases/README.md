# 启动界面工厂橱窗

每个子目录都是一份标准 Free 存档：

- `blocks.bin` 保存场景、系统块、工厂块和方块设置，不保存运行时材料。
- `meta.json` 可被普通 Free 存档读取，其中额外的 `showcase` 字段控制播放节奏和等轴相机构图。

## 可视化编辑

在仓库根目录执行：

```bash
scripts/showcase_author.sh edit welding
```

脚本第一次运行时会把资源复制为 `saves/showcase_author_welding`，随后直接用游戏打开。
像编辑普通 Free 世界一样移动、增删并配置方块，完成后在游戏中保存并退出。已有编辑副本不会被脚本覆盖。

确认修改后显式发布：

```bash
scripts/showcase_author.sh publish welding
```

发布只替换对应资源的 `blocks.bin`，不会覆盖 `meta.json` 的播放和相机配置。重新编译或运行游戏后即可在启动菜单预览。

可用场景 ID：`transport`、`conversion`、`welding`、`pusher`、`lifter`。

## 播放配置

可以直接修改对应目录的 `meta.json`：

- `turn_seconds`：每个模拟回合的展示间隔。
- `max_turns`：未完成时的最长回合数；材料成功验收会提前切换。
- `completion_hold_seconds`：成功验收后保持最终画面的时间，便于看清较短动作。
- `camera_position` / `camera_target`：等轴相机位置与观察点。
- `viewport_height`：正交视野的世界高度，越大看到的范围越广。

标题和副标题使用 `assets/i18n/*.json` 中同场景 ID 的 `showcase.*` 文案。
