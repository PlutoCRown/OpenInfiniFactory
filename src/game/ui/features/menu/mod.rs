mod actions;
pub mod types;

use bevy::prelude::*;

pub use actions::menu_actions;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(menu_actions);
    }
}
