# 代码结构与 UI 审查（临时记录）

初审日期：2026-09-10。实施日期：2026-09-11。基线：`d684afc`。状态：审查问题已实施，保留为阶段性设计记录。

本记录汇总 System / Plugin、分层和依赖方向审查，并补充 UI 设计与性能检查。原结构审查对 `src/`、`crates/` 下 414 个 Rust 文件、约 61,213 行做了静态盘点，重点沿调度、UI、编辑、会话和模拟表现调用链阅读；并非逐行验收全部源码。下文“问题与证据”保留初审时的文件位置，重构后行号会漂移。性能描述区分确定的重复工作与尚未测量的用户可见影响。

## 实施结果

| 编号 | 状态 | 实施结果 |
| --- | --- | --- |
| A1 | 已修复 | 删除 UI 的全局 `World` 裸指针与绕过借用的访问方式。普通系统显式取得 `UiContext`；挂载、确认框和输入框通过 `Commands` 排队的类型化 `UiRequest` 在安全时机执行。线程局部内容仅是调用栈内的只读文案/资产快照，作用域退出即恢复。 |
| A2 | 已修复 | 新增应用级 `GameSet` 顺序：菜单 → 会话 → 模拟控制 → 模拟 → 表现。`TurnCommitted` 携带 `SessionEpoch`，重置和回滚会更换身份，表现层拒绝旧会话或非当前回合消息。 |
| A3 | 已修复 | `GameSchedulePlugin` 定义业务阶段；`PerfPlugin` 只挂接观测标记，不再拥有业务正确性。新增不安装性能插件的顺序回归测试。 |
| A4 | 已修复 | `UiHost` 只管理通用挂载、模态框和完成回调；具体页面通过 `PanelMount` 构建回调接入。Settings、Save、SaveSettings、Inventory 使用各自的类型化 `UiAction<T>`。覆盖层关闭后由 `OverlayClosed` 通知功能插件处理背包业务。 |
| A5 | 已修复 | 编辑先更新 `WorldBlocks`/`StructureState`，渲染资产缺失只跳过表现刷新。`SceneRenderMut` 不再可变访问结构状态，并新增无渲染资产编辑回归。 |
| A6 | 已修复 | 原 `LocalPlayerMut` 按放置、悬停、输入、剪贴职责拆为窄 SystemParam；表现参数删除调试和结构写访问。会话换档仍使用聚合参数，因为它确实要原子重置整组会话/模拟状态。 |
| A7 | 已修复 | `placement_input` 只发出传送请求，玩家位移由独立消费系统执行；玩家/摄像机生成移到 cameras；模型生成移除调试与结构参数。保留仍属单一算法事务的长函数，避免产生一次调用的薄包装。 |
| A8 | 已修复 | `WorldBlocks::restore_cell` 统一处理分层恢复、附着、焊缝、计数与拓扑失效；撤销补丁只提供目标层和快照。全部内部集合、派生计数和版本对 crate 外私有，主程序只能通过只读 accessor 查询，写入由世界方法维护。`clear` 同时清理传送与旋转到达标记。 |
| A9 | 已修复 | simulation 的 structures、structure_state、movement 以及 UI systems 从共享作用域 `include!` 改为真实子模块；scene 直接依赖 `oif_sim::TurnOutput`，删除纯转发 `turn_visuals.rs` 与无效参数。shared/config 与 shared/save 仍作为强内聚的单一门面实现，强拆会增加大量跨模块可见接口，故保留。 |
| U1 | 已修复 | 删除通用按钮宏和 Start/Pause/Settings 的聚合业务上下文。按钮只绑定 `run_system_cached` 处理系统或发出会话请求；新增按钮依赖只出现在对应处理系统参数中。 |
| U2 | 已修复 | Generator/Goal 等面板按设置、语言、实体新增和资源变化更新；写 `Text`、`Node`、图标前比较新旧值。 |
| U3 | 已修复 | 虚拟遥控器拆开布局、按压外观和摇杆位移；空闲且布局键不变时不替换全部 `Node`，并有回归测试。 |
| U4 | 已修复 | 存档列表先做变化门控再构造行；顶层世界改为迭代器；悬停只更新前后实体，标签重算与样式更新分离。 |
| U5 | 已修复 | 天空盒字节使用共享不可变身份，预览按字节身份缓存 `Handle<Image>`；选择器和其他字段变化不再重复解码。有效 PNG 回归测试验证非空句柄复用。 |
| U6 | 已修复 | 背包只消费 `Changed<Interaction>`，仅更新旧/新悬停与选择项；tooltip 只在内容、语言或坐标真正变化时写组件。 |

这些改动减少了共享宏、纯转发层和重复不变量代码，也让新增菜单业务依赖不再扩大点击分发系统的参数集合，符合本次结构优化必须降低维护成本的前提。

## 缓存与内部状态复核

| 数据 | 语义 | 失效/更新方式 | 结论 |
| --- | --- | --- | --- |
| `WorldBlocks` | 权威世界事实；方块、设置、焊缝和附着都属于同一聚合 | 只能由世界编辑入口维护拓扑身份、计数和附属数据；撤销也走 `restore_cell` | 不是缓存，不再另建可写世界副本。`TurnCommitted.before` 只是单回合只读差分快照。 |
| `StructureState`、`PusherState`、`MovementHistory`、`PendingTurnEffects` | 模拟算法跨回合必须保留的内部状态 | 由回合 Schedule 与会话重置统一维护；世界换档和回滚同时清理/重建 | 应视为模拟内部状态，不对 UI 暴露为另一份业务事实。 |
| `SignalNetworkCache` | 从世界接线拓扑派生的真正缓存 | 用 `Arc` 拓扑身份判断；材料移动不会误失效，接线、面板和相关拓扑变化才更换身份 | 保留合理，失效频率与实际依赖一致。已有独立世界/快照分支测试。 |
| `GameplayStatusCache`、`InventorySlotRenderState`、`VirtualControlRenderState` | UI 渲染记忆，用来避免相同 `Text`/`Node`/样式重复写入 | 键只包含实际渲染依赖，并补齐语言、实体新增、图标就绪、视口/布局和交互变化 | 保留合理；不会成为业务真相，空闲帧能提前返回。 |
| `SaveSettingsSkyboxPreviewCache` | 图片字节身份到 Bevy `Handle<Image>` 的资源缓存 | 仅字节 `Arc` 身份或首次挂载变化时解码；选择器、灯光等字段不使其失效 | 保留合理，修复了近似“无缓存”的重复解码。 |
| GLB 加载内的 `material_cache` | 单次模型导入中复用同一材质索引 | 随导入函数结束释放 | 生命周期局部且没有跨帧一致性问题。 |

本次没有增加覆盖所有页面的统一缓存。需要跨回合延续的数据归入模拟内部状态；只为省重复渲染工作的数据保留为局部缓存，并让失效键直接对应其读取的输入。这分别处理了“多份事实可能漂移”“失效过密等于无缓存”和“本应是内部状态却以缓存命名”三个问题。

## 已确认的结构问题

| 编号 | 优先级 | 问题与证据 | 建议与收益 |
| --- | --- | --- | --- |
| A1 | P1 | `src/game/ui/access.rs:26` 保存全局 World 裸指针，`:41` 在普通系统里重建可变访问并通过 SystemState 应用命令。`pause_menu/mod.rs:217` 持有 UiNavigation 的可变访问，按钮回调又经 UiHostCommands 取得它。NonSendMarker 仅约束执行线程，不能证明独占访问。 | 移除全局 World 访问。通过显式系统参数、类型化 UI 请求和合法的延迟命令执行操作；减少隐藏依赖，恢复 Bevy 对访问冲突的检查。未运行复现具体崩溃。 |
| A2 | P1 | `src/game/session/mod.rs:88` 的会话操作链与 `src/game/mod.rs:283` 的模拟链均夹在 Menus/Simulation 之间，互相没有明确顺序。`src/sim_bridge/present.rs:32` 的 TurnCommitted 携带 before，但表现阶段读取当前 WorldBlocks 作为 after，缺少会话身份。 | 明确会话提交 → 模拟控制 → 推进 → 表现顺序；世界替换后让旧回合消息失效。静态调度允许旧回合与新世界混用，尚未运行复现。 |
| A3 | P2 | `src/game/systems/perf.rs:214` 定义业务阶段顺序；UI 直接依赖 perf_mark_ui_* 函数排序。部分局部 chain 没有必要的数据依赖，例如 apply_fov 与绘制包围盒。 | 应用层定义业务 SystemSet，性能插件只观测。移除与观测耦合的排序，保留真正的数据依赖；不意味着当前所有系统都逐一串行。 |
| A4 | P2 | `src/game/ui/core/host.rs:5` 依赖具体设置/存档页面和业务 Action；`core/panel.rs:51` 包含 SettingsTab；`systems/panels.rs:86` 在通用关闭流程中写快捷栏、清手持物品、标记解法 dirty。 | UI 基础层处理通用交互与挂载，功能层处理关闭后的业务。用结果消息或构建回调反转依赖，避免通用组件接收整组业务资源。 |
| A5 | P2 | `src/scene/scene_render.rs:11` 持有可变 StructureState；`scene/incremental.rs:660` 渲染刷新执行 apply_factory_edit；`game/session/world_access.rs:44` 无渲染资源就返回，使结构更新也被跳过。 | 编辑事务先维护世界和结构并输出变更，渲染随后消费。逻辑正确性不应依赖 GPU/渲染资产，渲染对结构只读。 |
| A6 | P2 | `src/game/local_player/mod.rs:41` 的 LocalPlayerMut 声明 8 个可变资源；悬停等系统因此取得不需要的写权限。SimulationPresentationDeps、PlayingWorldParams 同样过宽。 | 按真实共同职责打包，缩小资源集合并区分读写；减少调度冲突和测试装配负担。缩短签名不等于降低耦合。 |
| A7 | P2 | placement_input 约 603 行，混合输入、手势、撤销、渲染、音效、传送和面板；spawn_block_model 约 493 行；camera_move 约 317 行。位置分别为 `game/systems/gameplay/placement.rs:63`、`game/world/rendering/spawn.rs:391`、`game/player/controller.rs:212`。 | 按意图解析、事务提交、表现消费拆职责；模型按已有表现能力分派。长度是定位线索，不是机械拆 System 的依据。 |
| A8 | P2 | `crates/oif-sim/src/world/grid/mod.rs:31` 公开内部集合和派生状态；`src/game/edit_history/patch.rs:413` 重复实现删除、附着清理、计数和恢复，调用方还要手工使索引失效。 | 世界层提供批量编辑/恢复入口统一维护不变量；撤销栈留在业务层。上一轮补齐当前失效路径，尚未完成封装。 |
| A9 | P2 | 有 30 处 include!；structures 的 6 个包含文件共约 1,820 行但共享模块内部作用域。`scene/turn_visuals.rs:13` 原样转发 8 个参数；world_rebuild 与 incremental 仍传递不使用的 debug/structure 参数。scene 从 sim_bridge 导入核心 DTO 再导出，也造成不必要的反向引用。 | 删除无效参数和纯转发，再按真实接口建立子模块。DTO 直接依赖定义层。不要用更多文件掩盖相同作用域和循环依赖。 |

优先顺序：A1/A2 正确性 → A4/A5/A8 职责与不变量 → A6 参数权限 → A3/A7/A9 注册、拆分与清理。先降低依赖，再调整文件布局；重构应减少重复维护，或让接口更通用、更容易测试，而非只增加类型和层数。

## Plugin 拆分判断

| 现状 | 判断 |
| --- | --- |
| GameplayInputPlugin 只初始化资源，gather_gameplay_input 在 GamePlugin 注册 | 职责不闭合；将输入注册归入插件，或删除空壳。 |
| GamePlugin 集中初始化资源、设置归一化和逐系统排序 | 应作为组合入口，功能内部注册下放到相应插件。 |
| StartMenuPlugin/StartMenuMountsPlugin，PauseMenuPlugin/PlayingOverlaysPlugin | 按点击与挂载拆成 Plugin，生命周期分散；可在同一功能插件下保留多个系统。 |
| InventoryPlugin、SettingsPlugin、SavePlugin | 有独立资源、输入和视图职责，整体可保留；改善调度和依赖。 |
| 天空盒、传送门材质、阴影材质等插件 | 独立注册材质与渲染能力，短小不代表多余。 |

实施后 `GameplayInputPlugin` 自己注册输入聚合，`GameplayPlugin` 拥有玩法输入/悬停/放置资源与系统，`SimulationBridgePlugin` 拥有回合推进与表现资源；`GamePlugin` 保留为组合入口。Start/Pause 功能插件也各自注册点击和挂载同步，没有继续用只有一段生命周期的独立挂载 Plugin。

## 模拟 ECS 方向（用户明确要求一并保留）

初审时模拟为 Bevy Resource 加普通函数组成的回合流水线，没有独立回合 Schedule。独立 oif-sim crate 的依赖边界合理，没有发现依赖主程序 UI/渲染；但“资源使用 ECS”与“模拟阶段由 ECS 系统调度”是两回事。

现已由 GUI 和无头会话共同驱动一个可复用回合 Schedule，以准备、信号、运动、收尾四个真实阶段组织 System。宿主把资源所有权临时移入专用 ECS World，回合完成后归还，因此不复制权威世界且保持回合原子性。计算辅助函数仍是普通 Rust 函数，没有把每个算法步骤机械注册为 System。

测试覆盖 GUI/无头结果一致性、连续复用、回滚后继续推进以及会话身份失效。`simulate_turn` 只保留为一次性兼容入口，连续会话持有 `TurnRunner`。

## UI 专项检查

**结论：菜单参数扩散主要由当前接口设计导致，不是 ECS、Bevy 或 Rust 要求 UI 层取得所有业务资源。** Bevy 确实需要执行操作的系统声明资源访问，但创建按钮和转发点击的系统不必声明该操作涉及的所有资源。Rust 的显式借用使这种耦合暴露出来；改用全局 World 指针只是隐藏它，并引入 A1 的安全问题。

### U1：统一按钮回调上下文不断膨胀（P2，结构问题）

`src/game/ui/features/pause_menu/mod.rs:35` 的 PauseMenuCtx 包含 11 个字段。按钮表的所有回调统一为 `fn(&mut PauseMenuCtx, &mut Commands)`。`:217` 的 dispatch_pause_menu_clicks 必须提前声明整个上下文的资源，即使本次只点击“继续游戏”。新增按钮需要新资源时，需要连带修改上下文、系统参数、上下文装配与按钮逻辑。

具体业务泄漏在按钮表的 toggle_builder_mode 分支：它重置模拟、克隆世界快照、决定解法名称和存档种类、切换背包、设置出生点。其他“保存”等按钮已经能通过 `session::save_current_world(commands)` 发出请求；项目不是没有业务入口，而是同一页面混用“直接执行业务”和“发出请求”两种方式。

`src/game/ui/list_ui_config.rs:1` 也不是通用按钮描述宏：它识别 StartMenuButton、PauseMenuButton、SaveListToolbarButton、SettingsFooterButton 四种具体业务类型，并固定部分 visible/label 的资源类型。新增一种按钮表可能还要修改共享宏。`core/host.rs` 的 UiActionKind 又集中容纳各业务 Action，形成另一处修改汇聚点。实施后该宏已删除，功能动作改为各自的类型参数。

建议分成三个职责：

1. 基础 Button / Panel：处理布局、文案、启用、选中及激活结果，不接收业务世界。
2. 菜单功能层：将按钮绑定到本功能的类型化动作或 observer。读取绘制标签/显隐所需的少量只读状态，复杂展示可投影成小型视图数据；不复制整份业务状态成为第二事实源。
3. 业务层：处理切换模式、保存、重置等请求，声明实际需要的资源，维护事务规则。世界/背包/出生点的实现依赖只在这里变化。

两种合理实现都可以保留：按钮绑定一个普通 Bevy observer 系统，让 Bevy 注入该处理系统的依赖；或者按钮只发送具体功能的请求，由已有业务系统处理。无需为了每个按钮创建 Plugin，也无需再建囊括所有功能的总 Action 枚举。

概念数据流：`按钮激活 → ToggleBuilderMode 请求 → 会话层修改世界/背包/存档/出生点 → UI 根据可见状态刷新`。新增业务依赖时只改处理请求的业务系统；新增按钮本身仍需添加描述、绑定及必要的展示规则。这与 React 风格的 props / onClick 责任划分一致，但不要求引入虚拟 DOM。

确认框可以返回 Confirmed/Cancelled 或携带少量业务数据的结果，由功能层继续发请求。保留延迟命令中的合法 World 回调也是可行方案；禁止普通系统通过全局裸指针重新访问 World。回调只是执行位置的选择，不能替代职责边界。

### U2：属性面板静止时仍重复写 Text、Node 和图标（P2，确认存在重复工作）

`src/game/blocks/generator/ui.rs:374` 的 update_panel 只检查面板是否激活，没有检查配置变化；打开生成器后每帧赋值行 display，并重新构造/赋值模式、周期、偏移文字。`:494` 的 update_dropdowns 即使材料列表未展开，也在面板激活时遍历所有材料选项刷新图标，并更新 tooltip。代码注释因面板关闭时 run_if 不执行、Local 无法重置，而主动放弃了“已填充”判断。

这属于挂载生命周期处理不完整，不能靠每帧强制更新作为长期解决方案。应以新增实体、激活面板身份、当前配置值、资产变化为更新条件；面板重新挂载由 Added 标记或实例身份负责初始化。Text/Node/ImageNode 在确有变化时才写入。关闭状态的下拉选项不需要逐帧回填。

局部已经有不同水平的处理：Teleport 名称更新会比较新旧字符串，Goal 文案也比较值，但 Goal 的行 Node 仍无条件赋值。不要把所有方块面板一概说成每帧重写文字。

### U3：触控控件每帧重建布局和样式（P2，触控路径）

`src/game/ui/features/virtual_remote/update.rs:504` 的 apply_virtual_control_layout 在启用触控、未打开布局编辑器时每帧遍历全部控件。`spawn.rs:296` 的 apply_layout_to_node 直接替换整个 Node；摇杆布局、背景、边框、文字透明度也反复写入。输入是否变化没有作为整体布局更新条件。

建议分开静态布局、按下外观和摇杆位移：布局仅随挂载、配置/视口条件变化更新；按下状态只影响发生变化的控件；拖动摇杆仅更新摇杆位置。不能删除逐帧输入采样，但采样不应导致所有控件布局重写。该路径不用于 touch.enabled 为 false 的桌面模式。

### U4：存档列表先做全量工作再判断变化，悬停也计算整行文案（P2，随列表规模放大）

`src/game/ui/features/save/update.rs:34` 的 update_save_list_rows 在检查结构变化前，每帧调用 save_list_puzzle_rows、克隆名称和构造方案列表。`src/shared/save/types.rs:144` 的 top_level_worlds 每次筛选并分配 Vec。keys 缓存减少了实体重建，但没有避免上游列表构造。

`update_save_list_styles` 的 hover.is_changed 会刷新全部按钮；`:413` 对每个按钮先执行 button_view，然后才在 paint_labels 分支决定是否使用生成的文案。`view.rs:25` 的 button_view 会分别计算 label/kind/meta/favorite/enabled 等，多次调用 top_level_worlds 并查找条目。N 条顶层存档生成 O(N) 个按钮，每个按钮又多次 O(N) 扫描，悬停刷新存在 O(N²) 扫描/分配的结构；不是已经测量到的帧时间。

建议先区分列表结构、行内容和交互样式：数据/挂载变化时构造列表和视图；悬停只更新前后受影响实体；单行操作读取已定位的条目，而非每个属性重新扫描集合。列表增删当前会整组 despawn/recreate，规模大时再考虑按稳定 ID 增量维护或可视区渲染，不能未经测量先引入复杂虚拟化。

### U5：存档设置的无关状态变化会重新解码天空盒预览（P2，交互时的重工作）

`src/game/ui/features/save_settings/mod.rs:79` 以整个 SaveSettingsUiState.is_changed() 为条件，执行 PNG 解码和 images.add。`:158` 的 actions.rs 中打开/关闭方块选择器、修改灯光或过滤条件都会改变同一资源；因此“仅展开选择器”也可能重新解码同一张天空盒图片。

游戏场景天空盒与设置预览现在都使用共享不可变字节身份。设置预览仅在图片身份或实体挂载变化时解码并创建 Handle，打开选择器等状态变化不会触发解码。是否改善用户设备上的帧时间仍需运行测量，源码级修复不能据此宣称具体毫秒收益。

### U6：背包已经避免空闲全量内容刷新，但悬停更新仍偏粗（P2，次于前述路径）

`src/game/ui/features/inventory/render.rs:76` 已经跟踪 selected、hover、资源变化等并提前返回，这部分应保留。但它每帧会扫描 slot_query 查找悬停；悬停改变后遍历全部槽位，给背景/边框赋值，且该查询包括常驻但隐藏的背包槽。只在 Playing 状态运行，不以背包打开与否限制热栏更新是合理的；应进一步把热栏、打开的背包和发生交互变化的实体分开处理。

同文件 `update_item_tooltip:220` 的触控分支在未选中物品时仍每帧写 display=None，选中时重复写固定定位字段。应比较实际样式，区分跟随指针需要变化的坐标与不变布局。

### 引擎机制与当前代码的责任

已核对本机安装的 Bevy 0.19.0 源码：bevy_ui 的 `layout/mod.rs:110` 根据 Node/ContentSize 的 changed 状态同步布局；`widget/text.rs:280` 将 Text.is_changed() 作为重新建立文字测量的条件。Bevy 的可变解引用会标记变化，不会替所有业务赋值判断新旧语义是否相等。因此无条件写回相同 Text/Node 会使底层重新进入相关处理路径；不等于每次都会完整重排整棵 UI 树。

ECS 的显式访问声明、组件变化跟踪是正常机制。当前问题是把“每帧调用更新系统”写成“每帧重新赋值全部视图”，以及把“业务 handler 需要资源”扩散成“整个菜单共享这些资源”。这些都不是不可避免的 Bevy/Rust 限制。

### 上一轮已改善、当前源码仍保留的路径

| 路径 | 已做调整 | 本次判断 |
| --- | --- | --- |
| 状态栏 `ui/systems/status.rs:108` | 跟踪文案实际使用的模拟字段和块数量，不因 accumulator 每帧变化重组文字 | 已改善；不能外推为所有 UI 都修复。 |
| HUD `ui/systems/hud.rs:27` | 按实际可见性条件与新挂载实体更新 | 已改善。 |
| 模拟速度写入 | 新旧值不同才写 SimulationState.speed | 已改善。 |
| 生成预览实体 `sim_bridge/present.rs` | 增删对齐时才构造预览表，每帧只推进必要的生长比例 | 已改善；它不是生成器配置面板的文字/下拉更新。 |
| 场景天空盒 `world/rendering/skybox.rs` | 共享不可变字节并按身份判断变化，避免每帧复制和哈希 | 已改善；存档设置 UI 的图片预览仍有 U5。 |

设置页面已有按变化提前返回、仅重绘旧/新悬停按钮的实现；背包也已有内容更新门控。这些可作为局部参照，不建议重写全部 UI 框架，也不建议再加一层覆盖所有页面的缓存。

### 运行时性能验收建议

结构和源码级重复写入已经修复。若要量化设备上的收益，应在可复现场景中测量静止打开生成器面板、触控 HUD 无操作、不同规模存档列表的空闲/悬停/增删、仅打开存档设置选择器，以及反复开关背包。

运行验证应覆盖：静止打开生成器面板、触控 HUD 无操作、存档列表不同规模的空闲/悬停/增删、仅打开存档设置的方块选择器、反复开关面板。分别统计业务系统耗时、Text/Node 写入或 changed 数量、实体增删、图片解码次数及布局/文字阶段耗时；对可复现场景比较帧时间分位数，不能只看某个 PerfScope 的均值。

验收目标示例：配置不变且不处于动画中的面板无需反复改写文字；布局配置不变时触控 HUD 不应替换全部 Node；悬停单项不应重算所有行文案；图片身份不变时选择器开关不应触发解码；新增按钮的新业务依赖不应改变公共 Button/Panel 或无关按钮的参数列表。

## 验证边界

已运行 `cargo test -p oif-sim --lib --no-fail-fast`（41 项通过）与 `cargo test --lib --no-fail-fast`（8 项通过），覆盖回合执行器、回滚身份、分层恢复、应用调度、UI 空闲更新与天空盒缓存。已运行 `git diff --check`，并对改动 Rust 文件执行 rustfmt。按项目规范未运行 `cargo check`、`cargo run` 或真实窗口/设备性能采样，因此右键传送手感、GPU 渲染、动画观感和帧时间收益仍属于运行时验收范围。
