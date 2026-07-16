//! 游玩输入聚合：键盘鼠标与虚拟遥感写入同一状态

mod state;

pub use state::{gather_gameplay_input, ActionPulse, GameplayInputState};

use bevy::prelude::*;

use crate::shared::launch::LaunchOptions;
use crate::shared::touch_profile::TouchProfile;

pub struct GameplayInputPlugin;

impl Plugin for GameplayInputPlugin {
    fn build(&self, app: &mut App) {
        let force_touch = app
            .world()
            .get_resource::<LaunchOptions>()
            .is_some_and(|options| options.force_touch);
        app.insert_resource(TouchProfile::detect_with_force(force_touch))
            .init_resource::<GameplayInputState>();
    }
}