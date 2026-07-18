use bevy::prelude::*;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::update_material_slot_hover;
use crate::game::blocks::panels::register_all_panels;
use crate::game::ui::access::UiAccessScope;
use crate::game::ui::core::text_input::InlineTextEditState;

/// Block property panels update inside the global UI access window.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockPanelSystems;

pub struct BlockPanelsPlugin;

impl Plugin for BlockPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BlockPanelSystems.in_set(UiAccessScope))
            .insert_resource(OpenBlockPanelDropdown::default())
            .insert_resource(InlineTextEditState::default())
            .add_systems(Update, update_material_slot_hover.in_set(BlockPanelSystems));
        register_all_panels(app);
    }
}
