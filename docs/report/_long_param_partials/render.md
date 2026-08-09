# Agent-render

## Agent-Render 处方（只读）

对齐约定：`spawn_*`→**C5**；`rebuild_*`→**C1**（另加 `world` + 动画/通电等调用期参数，**不**新建刷新类型）。禁止手段2用于返回 `Entity` 的 spawn 路径。

### 逐函数处方

| 函数 | params | kind | 手段 | 共享簇ID | local-only | 风险 | P |
|---|---:|---|---|---|---|---|---|
| `spawn_block_model` | 17 | Opts+Mode | **1+3** | **[C5]** | 固定6：`commands,meshes,render_assets,world,pos,data`；Opts/Mode：`material,edit_preview,animation,pusher_animation,timing,with_block_entity/index,pending_generated_preview,show_generator_preview,icon_render,factory_debug`（含 powered 材质选择） | 多路径布尔互斥；`index`/`icon_render`/`factory_debug` 借用生命周期 | **P0** |
| `spawn_world_block_entity` | 12 | Opts 包装 | **1+3** | **[C5]** | `pos,data,animation,pusher,timing,powered_wire,factory_debug`→填 C5 | 与 `spawn_and_index`(SimPresent) 必须同 C5 | **P0** |
| `spawn_block_with_timed_animation` | 11 | Opts 包装 | **1+3** | **[C5]** | `animation,timing,factory_debug,powered_wire,index` | 薄包装勿再拆函数 | **P1** |
| `spawn_block_with_animation` | 9 | Opts 包装 | **1+3** | **[C5]** | `animation,factory_debug,index`（timing=edit） | selection 动画增量依赖 | **P1** |
| `setup_scene` | 14 | System 入口 | **3 accept** | `[]` | 全部 Bevy `Res*`/`Commands`（材料资产+registry+lighting） | 非 C1（无 index/chunks）；勿硬塞 SceneRenderMut | **P3** |
| `insert_configured_pack` | 13 | 装载 helper | **1 local** 或 **3** | `[]` | 见缺口；调用期：`kind,paths,fallback` | 仅 `WorldRenderAssets::new` 内循环；禁称 Spawn/Scene ctx | **P2** |
| `rebuild_world_with_runtime_animations_for_debug_state` | 12 | plain C1 | **1** | **[C1]** | `world` + `animations,pusher_animations,timing,powered_wires` | `_debug/_structure` 未用；仓库内几乎无外部调用 | **P1** |
| `rebuild_world_with_runtime_animations` | 11 | plain C1 | **1** | **[C1]** | `world` + `animations,pusher_animations,timing,powered_wires`；`factory_debug`↦C1.`structure_state` | 体内循环调 C5 核 | **P1** |
| `rebuild_world_with_animations_for_debug_state` | 9 | plain C1 | **1** | **[C1]** | `world` + `animations` | 同上未用 debug/structure 包装 | **P2** |
| `rebuild_world_with_timed_animations` | 9 | plain C1 | **1** | **[C1]** | `world` + `animations,timing` | edit_ops 经 `rebuild_world_with_animations` 使用 | **P1** |
| `append_scene_cube_faces` | 11 | 几何 helper | **3 accept** | `[]` | 缓冲5 + `pos,world,assets,face_uvs,origin,cull` | 纯同步网格写；勿消息化 | **P3** |
| `sync_goal_play_visual_on_builder_mode` | 10 | System | **1** | **[C1]** | `builder_mode, world, block_entities` | 有 `block_entities` 但无 history→**勿升 C2**；despawn/rebuild 吃 C1 | **P1** |

### 目标签名形态（不改码，仅处方）

- **C5 核：** `(commands, meshes, render_assets, world, pos, data, opts: SpawnBlockOpts, mode: SpawnMode) -> Entity`
- **C1 rebuild：** `(&mut SceneRenderMut, world: &WorldBlocks, /* animations/timing/powered… */) ` — C1 已含 `commands,meshes,render_assets,block_index,scene_chunks,debug,structure_state`

### 统计

| 项 | 值 |
|---|---|
| 清单函数 | 12 |
| 手段1（主） | 9（4 spawn + 4 rebuild + 1 sync） |
| 手段1+3（Opts/Mode） | 4（全在 spawn） |
| 手段3 accept | 2（`setup_scene`,`append_scene_cube_faces`） |
| 手段2 | **0** |
| 命中 C5 | 4 |
| 命中 C1 | 5（4 rebuild + sync） |
| 无注册表簇 | 3 |
| P0 / P1 / P2 / P3 | 2 / 6 / 2 / 2 |

### 缺口（只写字段列表，不命名）

1. **`insert_configured_pack` 装载汇：** `meshes, materials, images, scene_meshes, scene_face_uvs, scene_block_materials, block_materials, preview_materials`（≥3 次同组出现于 scene/material/stamp 循环）
2. **`append_scene_cube_faces` 网格属性缓冲：** `positions, normals, uvs, colors, indices`（若父级坚持降参；否则 P3 接受更合适）

### 跨领域对齐备忘

- SimPresent `spawn_and_index` → 同 **C5**（经 `spawn_world_block_entity`）
- rebuild / goal sync / 编辑刷新半截 → **C1**；需要世界数据时 **C1 + `world`**，不要 `SceneRefreshCtx`
- C1 与 C5 都含 `commands/meshes/assets`：spawn 路径只吃 C5，rebuild 持 C1 再构造 C5 调用，禁止平行第三套