mod actions;
mod confirm;
pub mod types;
mod update;

use bevy::prelude::*;

pub use actions::menu_actions;
pub use update::update_menu_labels;

use crate::game::systems::perf::PerfScope;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(menu_actions).add_systems(
            Update,
            update_menu_labels
                .after(crate::game::ui::update_localized_ui)
                .after(PerfScope::Animation)
                .before(PerfScope::Ui),
        );
    }
}
