mod actions;
mod render;
mod types;

use bevy::prelude::*;

pub use actions::{dispatch_inventory_slot_actions, emit_inventory_slot_actions};
pub use render::{
    update_carried_item_ui, update_inventory_slots, update_inventory_title,
    update_inventory_tooltip,
};
pub use types::InventoryTitleText;

use crate::game::state::{GameMode, PlayingUiState};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_inventory_slot_actions)
            .add_systems(
                Update,
                dispatch_inventory_slot_actions
                    .after(PerfScope::Placement)
                    .before(PerfScope::Menus),
            )
            // 热栏 / tooltip 常驻，不能绑 inventory_open，否则关背包后不刷新、tooltip 残留
            .add_systems(
                Update,
                (update_inventory_slots, update_inventory_tooltip)
                    .run_if(|mode: Res<State<GameMode>>| *mode.get() == GameMode::Playing)
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            )
            .add_systems(
                Update,
                update_inventory_title
                    .run_if(|playing_ui: Res<PlayingUiState>| playing_ui.inventory_open)
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            )
            .add_systems(
                Update,
                update_carried_item_ui
                    .in_set(UiAccessScope)
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
