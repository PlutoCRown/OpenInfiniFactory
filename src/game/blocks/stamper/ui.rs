use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::StamperBlock;

use crate::game::block_editing::widgets::{
    position_dropdown_from_trigger, spawn_text_dropdown_list, spawn_text_dropdown_toggle,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::StampColor;
use crate::game::edit_history::{apply_block_settings_with_history, EditHistory};
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{i18n, UiMainThread};
use crate::game::ui::components::{
    default_button_size, localized_text, spawn_panel_with_title_marker, transparent_node,
    PanelOptions,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::WorldBlocks;

const COLOR_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StamperAction {
    ToggleColor,
    SetColor(StampColor),
}

#[derive(Component, Clone, Copy)]
pub struct StamperPanelTitle;

#[derive(Component, Clone, Copy)]
struct StamperColorLabel;

#[derive(Component, Clone, Copy)]
struct StamperColorList;

impl UiActionLabel for StamperAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleColor | Self::SetColor(_) => "button.next_color",
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
            spawn_row(panel, "panel.color", |row| {
                spawn_text_dropdown_toggle(row, StamperAction::ToggleColor, StamperColorLabel);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_text_dropdown_list(
        root,
        StamperColorList,
        StampColor::ALL
            .into_iter()
            .map(|color| (i18n.t(color.name_key()), StamperAction::SetColor(color))),
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click).add_systems(
        Update,
        (update_title, update_dropdowns)
            .chain()
            .in_set(BlockPanelSystems),
    );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Stamper,
        spawn_panel: spawn_panel,
        spawn_overlays: spawn_overlays,
        register: register,
    }
}

fn spawn_row(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel
        .spawn(transparent_node(Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(40.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        }))
        .with_children(|row| {
            row.spawn((
                localized_text(label_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
            ));
            controls(row);
        });
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
        StamperAction::ToggleColor => {
            open_dropdown.toggle(UiPanelId::Stamper, COLOR_SLOT);
            return;
        }
        StamperAction::SetColor(color) => {
            settings.color = color;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_with_history(edit_history.as_mut(), &mut world.world, pos, |blocks| {
            blocks.set_stamper_settings(pos, settings);
        });
        solution_state.dirty = true;
        refresh_world_after_edit(&mut world, pos);
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
    for mut text in &mut titles {
        text.0 = i18n.t("stamper.title");
    }
}

fn update_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut labels: Query<(&StamperColorLabel, &mut Text)>,
    mut lists: Query<(&StamperColorList, &mut Node, &ComputedNode)>,
    triggers: Query<(&StamperAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Stamper;

    if let Some(pos) = ui_runtime.active_block_pos() {
        let label = i18n.t(world.stamper_settings(pos).color.name_key());
        for (_, mut text) in &mut labels {
            text.0 = label.clone();
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);

    for (_, mut style, list_node) in &mut lists {
        let open = open_dropdown.is_open(panel, COLOR_SLOT);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == StamperAction::ToggleColor && !node.is_empty()).then_some((node, transform))
        });
        if let Some((trigger_node, transform)) = trigger {
            if let Some((left, top)) =
                position_dropdown_from_trigger(trigger_node, transform, list_node, viewport)
            {
                style.left = Val::Px(left);
                style.top = Val::Px(top);
            }
        }
    }
}
