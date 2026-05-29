# Blocks

Total: 30

## SceneBlock

| 方块 | 中文名 | 可四向旋转 | 实现 Trait |
| --- | --- | --- | --- |
| Grass | 草方块 | ❌ | `Block`, `SceneBlock` |
| Stone | 石头 | ❌ | `Block`, `SceneBlock` |
| Dirt | 泥土 | ❌ | `Block`, `SceneBlock` |
| Planks | 木板 | ❌ | `Block`, `SceneBlock` |
| Glass | 玻璃 | ❌ | `Block`, `SceneBlock` |

## MaterialBlock

| 方块 | 中文名 | 可四向旋转 | 可被生成器选择 | 实现 Trait |
| --- | --- | --- | --- | --- |
| Material | 材料 | ❌ | ✅ | `Block`, `MaterialBlock` |
| Iron Material | 铁材料 | ❌ | ✅ | `Block`, `MaterialBlock` |
| Copper Material | 铜材料 | ❌ | ✅ | `Block`, `MaterialBlock` |

## FactoryBlock

| 方块 | 中文名 | 可四向旋转 | 替换方块 | 实现 Trait |
| --- | --- | --- | --- | --- |
| Solid | 工厂方块 | ❌ |  | `Block`, `FactoryBlock` |
| Welder | 焊接器 | ✅ | Down Welder | `Block`, `FactoryBlock` |
| Down Welder | 向下焊接器 | ❌ | Welder | `Block`, `FactoryBlock` |
| Conveyor | 传送带 | ✅ | Reverse Conveyor | `Block`, `FactoryBlock` |
| Reverse Conveyor | 反向传送带 | ✅ | Conveyor | `Block`, `FactoryBlock` |
| Detector | 方块识别器 | ✅ | Down Detector | `Block`, `FactoryBlock` |
| Down Detector | 向下方块识别器 | ❌ | Detector | `Block`, `FactoryBlock` |
| Wire | 电线 | ❌ |  | `Block`, `FactoryBlock` |
| Piston | 活塞 | ✅ | Blocker | `Block`, `FactoryBlock` |
| Lifter | 抬升器 | ✅ |  | `Block`, `FactoryBlock` |
| Rotator | 旋转器 | ✅ | Counter Rotator | `Block`, `FactoryBlock` |
| Counter Rotator | 逆向旋转器 | ✅ | Rotator | `Block`, `FactoryBlock` |
| Blocker | 阻拦器 | ✅ | Piston | `Block`, `FactoryBlock` |
| Drill | 钻头 | ✅ | Laser | `Block`, `FactoryBlock` |
| Laser | 激光 | ✅ | Drill | `Block`, `FactoryBlock` |

## SystemBlock

| 方块 | 中文名 | 可四向旋转 | 编辑模式可放置 | 实现 Trait |
| --- | --- | --- | --- | --- |
| Generation Block | 生成块 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` |
| Acceptance Block | 验收块 | ❌ | ✅ | `Block`, `SystemBlock`, `EditableBlock` |
| Stamper | 印花器 | ✅ | ✅ | `Block`, `SystemBlock`, `EditableBlock` |
| Roller | 滚刷器 | ✅ | ✅ | `Block`, `SystemBlock`, `EditableBlock` |
| Weld Point | 焊接点 | ❌ | ❌ | `Block`, `SystemBlock` |
| Blocker Head | 阻拦头 | ❌ | ❌ | `Block`, `SystemBlock` |
| Drill Head | 钻头头 | ❌ | ❌ | `Block`, `SystemBlock` |
