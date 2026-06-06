mod actions;
mod confirm;
pub mod types;
mod update;

use bevy::prelude::*;

pub use actions::{dispatch_menu_actions, emit_menu_actions};
pub use update::update_menu_labels;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_menu_actions).add_systems(
            Update,
            (
                dispatch_menu_actions
                    .in_set(UiAccessScope)
                    .after(PerfScope::Input)
                    .before(PerfScope::Menus),
                update_menu_labels
                    .in_set(UiAccessScope)
                    .after(crate::game::ui::update_localized_ui)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            ),
        );
    }
}
