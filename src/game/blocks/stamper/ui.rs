use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use super::StamperBlock;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::color_slot_ui::{
    self, ColorSelectOption, spawn_color_select_list, spawn_color_select_row,
};
use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{StampMaterialId, stamp_catalog};
use crate::game::edit_history::EditHistory;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::{PanelOptions, spawn_panel_with_title_marker};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StamperAction {
    ToggleStamp,
    SetStamp(StampMaterialId),
}

#[derive(Component, Clone, Copy)]
pub struct StamperPanelTitle;

impl UiActionLabel for StamperAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleStamp | Self::SetStamp(_) => "button.next_color",
        }
    }
}

impl BlockUi for StamperBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Stamper)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel_with_title_marker(
        root,
        PanelOptions::new(420.0, "stamper.title").closable(),
        UiPanelBinding(UiPanelId::Stamper),
        StamperPanelTitle,
        |panel| {
            spawn_color_select_row(panel, StamperAction::ToggleStamp);
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_color_select_list(
        root,
        stamp_catalog()
            .iter()
            .map(|(id, _)| (id, StamperAction::SetStamp(id))),
        ColorSelectOption::Stamp,
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click)
        .add_systems(Update, update_title.in_set(BlockPanelSystems));
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Stamper,
        spawn_panel: spawn_panel,
        spawn_overlays: spawn_overlays,
        register: register,
    }
}

fn on_click(
    mut click: On<Pointer<Click>>,
    ui_host: Res<UiHost>,
    ui_runtime: Res<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&StamperAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Stamper) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let mut settings = world.world.stamper_settings(pos);
    let changed = match action {
        StamperAction::ToggleStamp => {
            open_dropdown.toggle(UiPanelId::Stamper, color_slot_ui::COLOR_SLOT);
            return;
        }
        StamperAction::SetStamp(stamp) => {
            settings.stamp = stamp;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_edit(edit_history.as_mut(), &mut world, pos, |blocks| {
            blocks.set_stamper_settings(pos, settings);
        });
        solution_state.dirty = true;
    }
}

fn update_title(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    mut titles: Query<&mut Text, With<StamperPanelTitle>>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Stamper) {
        return;
    }
    let title = i18n.t("stamper.title");
    for mut text in &mut titles {
        if text.0 != title {
            text.0 = title.clone();
        }
    }
}
