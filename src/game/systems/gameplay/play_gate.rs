//! 玩法输入门控（权威簇 C7）
//!
//! PlayingUi 属本地玩家会话，由调用方传入，避免与 LocalPlayerMut 争用同一 Resource。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::state::{GameMode, SimulationState};
use crate::game::ui::UiNavigation;

/// Playing 态输入门控：模式 / UI 挡操作 / 模拟是否在跑
#[derive(SystemParam)]
pub struct GameplayPlayGate<'w> {
    pub mode: Res<'w, State<GameMode>>,
    pub ui_navigation: Res<'w, UiNavigation>,
    pub simulation: Res<'w, SimulationState>,
}

impl GameplayPlayGate<'_> {
    /// Playing 且未暂停，且 UI 未挡住玩法
    pub fn allows_active_play(&self) -> bool {
        *self.mode.get() == GameMode::Playing && self.ui_navigation.active_play()
    }

    /// 允许改世界（active play 且模拟未跑）
    pub fn allows_world_edit(&self) -> bool {
        self.allows_active_play() && !self.simulation.is_active()
    }
}
