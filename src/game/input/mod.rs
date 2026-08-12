//! 游玩输入聚合：键盘鼠标与虚拟遥感写入同一状态

mod state;

pub use state::{ActionPulse, GameplayInputState, gather_gameplay_input};

use bevy::prelude::*;

pub struct GameplayInputPlugin;

impl Plugin for GameplayInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayInputState>();
    }
}
