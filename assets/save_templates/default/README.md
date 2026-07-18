# 默认新存档模板

新建谜题时会把本目录整份复制到 `saves/<name>/`。

| 文件 | 作用 |
|------|------|
| `meta.json` | 存档元数据（光照等），见 `schemas/save.meta.schema.json` |
| `blocks.bin` | 默认世界（草地地板） |
| `skybox.png` | 水平十字天空盒 |

改模板后无需改代码；桌面运行时优先读磁盘本目录，嵌入二进制为兜底。
