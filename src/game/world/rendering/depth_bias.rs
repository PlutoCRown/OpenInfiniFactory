//! 共面叠层用的 `depth_bias` 约定（详见 `docs/spec/depth_bias.md`）
//!
//! 普通方块本体为隐式 `0`（`StandardMaterial` 默认），此处只列需显式写入的档位。
//!
//! Bevy 会把该值 `(as i32)` 写入 `wgpu::DepthBiasState::constant`（深度纹素单位），
//! 同时半透明排序用 `distance + depth`。`1`/`2` 在 GPU 上几乎看不出，需用百～千量级。

/// 验收器游玩态目标材料虚影（画在本体「后面」一侧，避免压过真材料）
pub const GOAL_GHOST: f32 = -1000.0;

/// 滚刷漆等零厚度面贴片（灯面板用默认 0，勿套此档）
pub const PAINT: f32 = 1000.0;

/// 瞄准面高亮、选区/删除框、结构悬停线框等编辑叠层
pub const OVERLAY: f32 = 2000.0;

/// Bevy `GizmoConfig::depth_bias` 取值在 `[-1, 1]`：越负越靠前。
/// 结构悬停线框对应叠层语义（等同 [`OVERLAY`]），用略负值压过方块避免 z-fight。
pub const GIZMO_OVERLAY: f32 = -0.0001;
