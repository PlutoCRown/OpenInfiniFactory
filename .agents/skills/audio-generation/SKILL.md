---
name: audio-generation
description: Generate, process, validate, and replace OpenInfiniFactory music and sound-effect assets with NumPy, SciPy, Pedalboard, and Pyo. Use for creating procedural placeholder or final audio, designing spatial game sound effects, editing existing WAV assets, or wiring newly generated audio into the game's audio registry.
---

# OpenInfiniFactory 音频生成

为游戏生成可重复、可调参的音乐和音效。优先保留生成源代码，生成的音频只作为可替换构建产物提交到游戏资源目录。

## 固定目录

所有路径均相对于仓库根目录 `C:\Users\Pluto_C\Desktop\OpenInfiniFactory`：

- 生成源文件：`tools/audio_generation/`
- 生成结果：`assets/audio/`
- 游戏运行时音频加载入口：`src/game/audio.rs`
- 现有旧占位生成器：`tools/generate_audio_placeholders.py`

新建或修改音频时，把 Python 源文件放到 `tools/audio_generation/`，例如：

```text
tools/audio_generation/machine_sounds.py
assets/audio/machine_motor.wav
assets/audio/machine_work.wav
```

不要把生成的 WAV 放进 Skill 自己的 `assets/` 目录；`assets/audio/` 才是本项目运行时资源目录。不要只提交无法复现的 WAV，除非用户明确要求只提交最终素材。

## 库的职责

先检查当前 Python 环境是否安装了所需库；不要因为某一个库缺失就修改项目依赖或全局 Python 环境。根据任务选择最小组合：

- `numpy`：采样数组、振荡器、包络、混音、归一化和批量生成的基础工具。
- `scipy`：滤波器、频谱处理、重采样、包络平滑和分析工具。
- `pedalboard`：离线后期处理，例如压缩、EQ、混响、延迟、失真和限制器。
- `pyo`：合成器、调制器和复杂信号链；需要离线交付时，必须录制或导出成 WAV，不要依赖运行时启动 Pyo server。

推荐的职责分工是 `numpy` 生成原始声音，`scipy` 整形信号，`pedalboard` 做后期，`pyo` 只在确实需要复杂合成或调制时使用。简单的咔哒声、机械声和占位音效不需要强行使用四个库。

## 生成工作流

1. 先查看 `src/game/audio.rs` 和 `assets/audio/`，确认音频文件名、触发点和是否为空间音频。
2. 在 `tools/audio_generation/` 创建可重复执行的脚本。把音高、时长、衰减、噪声量、滤波器和输出文件名写成顶部常量或参数，不要把大量不可解释的采样数据硬编码进源码。
3. 为每个声音生成明确的 WAV 输出到 `assets/audio/`。脚本重复运行应覆盖同名开发产物，并保持文件名稳定。
4. 对游戏方块、机器和世界事件使用单声道音效，让 Bevy 的 `SpatialAudioSink` 负责方向和距离定位；音乐和明确的 UI 非空间声音可以使用立体声。
5. 生成后验证采样率、位深、声道数、时长、峰值和文件头；确认没有 NaN、无穷值、削波或明显直流偏移。
6. 将新文件接入 `src/game/audio.rs` 或对应的配置/资源注册表，并确保事件使用的 `SoundId` 与文件名一一对应。
7. 用 `git diff --check` 检查文本差异；不要用全仓库格式化命令覆盖无关代码。除非用户明确要求运行游戏，否则不要执行 `cargo check` 或 `cargo run`。

## 推荐音频规格

- 空间音效：单声道 WAV，22050 Hz 或 48000 Hz，16-bit PCM。
- 音乐：需要立体声时使用 44100/48000 Hz、16-bit PCM；没有立体声需求时也可单声道。
- 一次性音效：从零振幅开始并在末尾淡出，避免点击声和尾部截断。
- 循环底噪：首尾尽量无缝，或在运行时使用足够短的交叉淡化；固定循环声不要包含突兀的最终衰减。
- 输出峰值保留约 1 dB headroom，避免多个空间音源叠加后削波。
- 先生成干净的源信号，再在最后一步统一增益、限制峰值和导出 WAV。

## 库可用性和降级

运行脚本前可用下面的方式检查环境：

```powershell
python -c "import numpy, scipy, pedalboard, pyo; print('audio libraries available')"
```

如果 `pedalboard` 或 `pyo` 不可用：

- 能用 NumPy/SciPy 完成时，删除不必要的导入并继续生成。
- 需要特定库的效果时，明确报告缺少的库，不要伪造效果已经应用。
- 不能因为生成占位素材而把这些库加入 Rust/Cargo 依赖；它们属于离线 Python 工具链。

## 空间音频注意事项

不要在音频文件里预先烘焙左右声道方向或距离衰减。游戏会根据方块的世界坐标和玩家监听器实时处理方向与距离；生成器只负责让声音本身具有清晰的起音、频段和动态。

机器持续底噪应提供稳定的循环版本；一次性动作如焊接、破碎、放置、验收和传送应使用短音效。需要多样性时，可以在生成脚本中输出多个变体，但必须在运行时随机选择，不能把多个变体同时作为同一个空间源播放。

## 交付检查

完成后确认：

- 源脚本位于 `tools/audio_generation/`。
- 运行结果位于 `assets/audio/`，并且文件名与加载入口一致。
- 生成结果能被 Bevy 当前启用的 WAV loader 读取。
- 空间音效是单声道且没有预烘焙方向。
- 生成脚本可再次运行并得到同样的文件集合；随机噪声使用固定种子或明确接受随机差异。
- 临时频谱图、试听导出和缓存文件不提交到仓库。
