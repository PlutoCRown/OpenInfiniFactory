/// 单回合模拟各阶段耗时采样（HUD / debug 采样）
#[derive(Clone)]
pub struct SimulationStepStats {
    pub has_sample: bool,
    pub total_ms: f64,
    pub prep_ms: f64,
    pub gravity_ms: f64,
    pub signal_ms: f64,
    pub marker_before_move_ms: f64,
    pub movement_mark_ms: f64,
    pub movement_execute_ms: f64,
    pub marker_after_move_ms: f64,
    pub behavior_ms: f64,
    pub signal_refresh_ms: f64,
    /// 仅由 scene 放映层写入；`simulate_turn` 不设置
    pub render_rebuild_ms: f64,
    /// 动画表转换 + 移动实体挂动画 / 重建
    pub render_anim_ms: f64,
    /// 传送瞬移或挂延迟
    pub render_teleport_ms: f64,
    /// 收集刷新格
    pub render_collect_ms: f64,
    /// 按格 despawn / spawn 实体
    pub render_refresh_ms: f64,
    /// 场景 chunk 合并 mesh 重建
    pub render_scene_ms: f64,
    /// 补漏材料 / 系统实体
    pub render_fill_ms: f64,
    /// 焊接 / 激光 / 破碎 / 验收特效
    pub render_fx_ms: f64,
}

impl Default for SimulationStepStats {
    fn default() -> Self {
        Self {
            has_sample: false,
            total_ms: 0.0,
            prep_ms: 0.0,
            gravity_ms: 0.0,
            signal_ms: 0.0,
            marker_before_move_ms: 0.0,
            movement_mark_ms: 0.0,
            movement_execute_ms: 0.0,
            marker_after_move_ms: 0.0,
            behavior_ms: 0.0,
            signal_refresh_ms: 0.0,
            render_rebuild_ms: 0.0,
            render_anim_ms: 0.0,
            render_teleport_ms: 0.0,
            render_collect_ms: 0.0,
            render_refresh_ms: 0.0,
            render_scene_ms: 0.0,
            render_fill_ms: 0.0,
            render_fx_ms: 0.0,
        }
    }
}
