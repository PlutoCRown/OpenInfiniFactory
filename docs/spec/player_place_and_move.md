# 玩家移动与放置逻辑

本文描述当前（实现时点）玩家**移动 / 飞行**与**方块瞄准、线面放置**的细节。实现分散在：

| 区域 | 路径 |
|------|------|
| 移动 / 碰撞 / 视角 | `src/game/player/controller.rs` |
| 输入采集 | `src/game/input/state.rs` |
| 瞄准与悬停 | `src/game/systems/gameplay/hover.rs` |
| 放置 / 删除手势 | `src/game/systems/gameplay/placement.rs` |
| 点/线/面展开 | `src/game/systems/gameplay/selection.rs` |
| 拖拽终点求交 | `crates/oif-sim/src/world/grid/raycast.rs` |

配置项：`place_selection_mode` / `delete_selection_mode`（点 / 线 / 面），见设置与 `GameConfig`。

---

## 一、玩家移动

### 1.1 角色与相机

- 组件 `FlyCamera` 挂在第一人称相机上；`Transform.translation` 即**眼睛位置**。
- 碰撞体相对眼睛：高度 `PLAYER_HEIGHT = 1.7`，半径 `PLAYER_RADIUS = 0.28`，眼睛约在胶囊顶部。
- 出生眼睛高度约 `SPAWN_EYE_Y`（地板顶 `1.0` + 眼高 + 余量）；下落不会低于该高度（硬钳）。

### 1.2 输入与水平移动

- 水平方向只吃 yaw：`forward = yaw * -Z`，`right = yaw * X`，由 `move_axis`（键鼠 ∪ 虚拟遥感）合成后归一化。
- 步行速度 `PLAYER_SPEED = 5.5`；飞行水平 `FLY_SPEED = 7.0`。
- 位移经 `move_with_collision`：先 X、再 Z、再 Y（若有），可与方块 / 活塞伸出体 / 场景碰撞块相交检测。

### 1.3 步行：重力、跳跃、上台阶

- 非飞行时：`velocity_y -= GRAVITY * gravity_scale * dt`（`GRAVITY = 18`），再竖直碰撞。
- 落地：竖直位移被挡住且 `velocity_y ≤ 0`，或 `is_supported` 为真 → `grounded`。
- 跳跃：着地时按 Jump；初速 `sqrt(2 * g * scale * 1.5)`，使眼睛大约能抬高 1.5（上到下一整格台面）。
- 水平撞墙时若 `allow_step_up`：在 `STEP_HEIGHT = 0.55` 内二分最小抬升后前进（半砖 / 斜面）；整格高墙仍需跳。

### 1.4 飞行

- **双击** Jump（间隔 ≤ `0.28s`）切换飞行；清空竖直速度与漂移状态。
- 飞行中：Space（`fly_up`）上升、默认 Shift（`fly_down`）下降，速度同 `FLY_SPEED`。
- 飞行中持续下潜且竖直被挡或脚下有支撑 → 退出飞行并着地。

### 1.5 飞行水平惯性（漂移）

仅在**飞行**且松开**水平移动键**时触发：

| 常量 | 值 | 含义 |
|------|-----|------|
| `FLY_DRIFT_DURATION` | `0.5` | 惯性持续时间（秒） |
| `FLY_DRIFT_DISTANCE` | `0.5` | 惯性总位移约 0.5 格 |

算法：

1. 按住 WASD 时记录 `fly_last_dir`（水平单位向量），并清零漂移计时。
2. 检测到「上一帧在移、本帧不移」：
   - 若松手瞬间按着 **Shift**（任意左右 Shift）→ **不漂**；
   - 否则启动漂移，`fly_drift_remaining = 0.5`。
3. 漂移中速度线性减速：初速 `v0 = 2d/T`，当前速度 `v0 * (remaining/T)`，积分位移仍为 `d`；结束时速度为 0（不会匀速掐断）。
4. 再按移动键、退出飞行、落地退出飞行时取消漂移。

默认 `fly_down` 也绑在 Shift 上：按住 Shift 下潜时松开 WASD，同样不漂。

### 1.6 视角

- `camera_look`：鼠标 / 虚拟摇杆改 yaw、pitch；pitch 钳在约 ±1.45。
- 按住 Alt 或 UI 挡住玩法时不转视角；触控不锁鼠。

---

## 二、瞄准与放置入口

### 2.1 瞄准射线

每帧 `update_hover`：

- 原点 = 相机位置，方向 = 相机 forward。
- **无手势**：`raycast_blocks` —— 对 `blocks` / `system_blocks` 做 AABB，取最近命中，得到 `TargetHit { pos, normal }`（`REACH = 12`）。
- **放置/删除手势中**（且模式为线/面）：改为 `raycast_edit_drag_grid`（见第三节），结果写入 `target.pos`，`normal = 0`。
- **点选模式手势中**：仍用 `raycast_blocks`。

### 2.2 放置格 vs 删除格

- `current_place_at`：
  - 点选：`target.pos + target.normal`（贴在瞄准面外侧那一格）；
  - 线/面拖拽中：`target.pos`（拖拽终点格，已是放置空间坐标）。
- `current_delete_at`：始终 `target.pos`。
- 手势起点 `EditGesture.start`：按下时的 `current_place_at` / `current_delete_at`。
- `plane_normal`：按下时瞄准面法线（由宿主指向放置格）；无面时默认 `Y`。

### 2.3 手势生命周期（摘要）

1. 按下放置 / 删除 → 创建 `EditGesture`（可被对侧重按取消）。
2. 拖拽期间 hover 更新终点；预览按点/线/面展开。
3. 松开对应键 → `commit_edit_gesture`：用 `selection_positions(mode, start, end)` 展开格子，再按规则过滤后写入世界。

配置：`place_selection_mode` / `delete_selection_mode` ∈ { Point, Line, Plane }。

### 2.4 点 / 线 / 面展开（`selection_positions`）

| 模式 | 行为 |
|------|------|
| Point | 仅 `start` |
| Line | 在 `|Δ|` 最大的轴上，从 start 扫到 end（另两轴锁在 start） |
| Plane | **当前实现**在 `y = start.y` 的水平面上，对 x、z 做矩形填充 |

注意：拖拽终点求交（第三节）在「附着竖直侧面」时会把终点锁在法线轴上并允许改 y/z；但 `plane_selection` 展开仍固定 `start.y`、扫 x/z。若面选贴在竖直墙上，实际铺开的格子与「YZ 墙面」直觉可能不一致——以本实现为准。

---

## 三、线 / 面拖拽终点：`raycast_edit_drag_grid`

核心在 `oif-sim`：`raycast_edit_drag_grid(origin, dir, start, mode, plane_normal)`。

目标：在**起始格**周围用射线与轴对齐平面求交，选出焦点，再吸附成线选或面选的终点格。

### 3.1 起始格的六张轴对齐面

对每个轴 `axis ∈ {0,1,2}`，起始格有 `min = start[axis]`、`max = min+1` 两张无限平面。射线与平面求交得参数 `t`（需 `t ∈ (ε, REACH]`）。

**近面 / 远面**（相对玩家）：

```text
center = min + 0.5
far    = (plane - center) * (origin[axis] - center) < 0
```

即：平面与玩家分居格心两侧 → 远面；同侧 → 近面。

### 3.2 焦点优先级（5 格惩罚）

每个合法交点得分：

```text
score = t - (far ? 5 : 0)    // FAR_FACE_PRIORITY_PENALTY = 5
```

取 **score 最大** 的交点作为焦点。含义：远面交点必须比近面交点大约再远 **5 格**，才能夺权。

### 3.3 过滤 A：附着法线内侧的面

若 `plane_normal[axis] ≠ 0`（放置贴在某一侧）：

- 法线朝 `+axis` → 丢掉 `min` 面（朝宿主内侧）；
- 法线朝 `-axis` → 丢掉 `max` 面。

例：瞄准方块**西面**放置，`plane_normal = (-1,0,0)`，起始格在宿主西侧 → **不与起始格东侧面求交**，避免线/面往东（回宿主）拉。

### 3.4 过滤 B：起始格虚拟立方体挡射线

把起始格当作实心 AABB `[start, start+1)`：

1. 对射线做 AABB 求交，得到穿出参数 `t_exit`（未命中则视为 `REACH`）；
2. 凡平面交点 `t > t_exit + ε` **全部丢弃**。

用途：视线几乎贴轴、会「穿过」首格再打到平面在后方的延伸时，不再把焦点选到被首格挡住的后方，避免拉出藏在起始块后面的一长串。

### 3.5 吸附成终点格

焦点 `(point, hit_axis)` 之后：

**面选** `snap_plane_on_normal`：

- 法线锁死该轴坐标为 `start`；
- 另两轴取 `world_to_grid(point)`。

**线选**：

1. `raw = world_to_grid(point)`，并把 `raw[hit_axis] = start[hit_axis]`；
2. `delta = raw - start`；取 `|Δ|` 最大轴；
3. `snap_line_on_plane`：只沿该轴走到 `grid`，另两轴锁 `start`。

**过滤 C：法线反向钳制**

对终点 `end` 每一轴：若 `(end[axis]-start[axis]) * plane_normal[axis] < 0`，则 `end[axis] = start[axis]`。  
防止线选结果仍沿法线反方向越过起点（拉回宿主一侧）。

### 3.6 流程简图

```text
相机射线
    │
    ├─ 虚拟立方体(start) → t_exit（挡穿透）
    │
    └─ 每轴 min/max 平面求交
           │
           ├─ 跳过内侧面（相对 plane_normal）
           ├─ 丢弃 t > t_exit
           ├─ 近/远打标，score = t - 5·far
           └─ 取 max score 焦点
                    │
                    ├─ Plane → 锁法线轴
                    └─ Line  → 主轴吸附
                    │
                    └─ 钳制：禁止沿 -normal 越过 start
```

### 3.7 其它调用

- **选区工具拖拽移动/复制**：与放置相同走 `raycast_edit_drag_grid`。
  - 起点 = 按下时抓住的选区内格；法线 = 当时瞄准面（无则 `Y`）。
  - 模式跟 `place_selection_mode`（若为 Point 则退化为 Plane）。
  - 选区整体平移 `offset = 终点 − 起点`，使抓住的那块落到终点格。

---

## 四、常量速查

### 移动（`controller.rs`）

| 常量 | 值 | 用途 |
|------|-----|------|
| `PLAYER_SPEED` | 5.5 | 步行水平 |
| `FLY_SPEED` | 7.0 | 飞行水平 / 竖直 |
| `GRAVITY` | 18 | 重力基数 |
| `ONE_BLOCK_JUMP_HEIGHT` | 1.5 | 跳起眼睛升高目标 |
| `STEP_HEIGHT` | 0.55 | 上台阶最大抬升 |
| `DOUBLE_TAP_WINDOW` | 0.28 | 双击切飞行 |
| `FLY_DRIFT_DURATION` | 0.5 | 飞行惯性时间 |
| `FLY_DRIFT_DISTANCE` | 0.5 | 飞行惯性路程 |
| `PLAYER_RADIUS` | 0.28 | 碰撞半径 |
| `EYE_HEIGHT` / `PLAYER_HEIGHT` | 1.7 | 眼高 / 胶囊高 |

### 放置求交（`raycast.rs` / grid）

| 常量 | 值 | 用途 |
|------|-----|------|
| `REACH` | 12 | 瞄准与拖拽最远距离 |
| `FAR_FACE_PRIORITY_PENALTY` | 5 | 远面焦点距离惩罚 |

---

## 五、相关文件索引

- 移动：`src/game/player/controller.rs`（`camera_move` / `camera_look` / `move_with_collision`）
- 输入：`src/game/input/state.rs`（`gather_gameplay_input`）
- 瞄准：`src/game/systems/gameplay/hover.rs`（`update_hover`）
- 手势：`src/game/systems/gameplay/placement.rs`（`placement_input` / `commit_edit_gesture`）
- 展开：`src/game/systems/gameplay/selection.rs`（`selection_positions`）
- 求交：`crates/oif-sim/src/world/grid/raycast.rs`（`raycast_edit_drag_grid` / `raycast_blocks`）
