use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::SignBlock;

use crate::game::block_editing::widgets::{
    position_dropdown_from_trigger, spawn_labeled_panel_button, spawn_material_icon_list,
    spawn_material_icon_toggle, update_material_icon,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::MaterialKind;
use crate::game::edit_history::{apply_block_settings_with_history, EditHistory};
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{i18n, UiMainThread};
use crate::game::ui::components::{
    default_button_size, localized_text, spawn_panel as spawn_ui_panel, text, transparent_node,
    PanelOptions,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::game::ui::access::ui;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::{SignDisplay, WorldBlocks};
use crate::game::world::rendering::BlockIconAssets;

const DISPLAY_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignAction {
    EditText,
    ClearText,
    ToggleDisplay,
    SetMaterial(MaterialKind),
    ClearDisplay,
}

#[derive(Component, Clone, Copy)]
struct SignTextPreview;

#[derive(Component, Clone, Copy)]
struct SignDisplaySlot;

#[derive(Component, Clone, Copy)]
struct SignDisplayList;

#[derive(Component, Clone, Copy)]
struct SignMaterialOption(MaterialKind);

impl UiActionLabel for SignAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::EditText => "button.sign_edit_text",
            Self::ClearText => "button.sign_clear_text",
            Self::ToggleDisplay | Self::SetMaterial(_) => "button.sign_display",
            Self::ClearDisplay => "button.sign_clear_display",
        }
    }
}

impl BlockUi for SignBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Sign)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_ui_panel(
        root,
        PanelOptions::new(460.0, "sign.title").closable(),
        UiPanelBinding(UiPanelId::Sign),
        |panel| {
            spawn_row(panel, "panel.sign_text", |row| {
                spawn_labeled_panel_button(row, SignAction::EditText);
                row.spawn((text("-", 16.0, Color::WHITE), SignTextPreview));
                spawn_labeled_panel_button(row, SignAction::ClearText);
            });
            spawn_row(panel, "panel.sign_display", |row| {
                spawn_material_icon_toggle(row, SignDisplaySlot, SignAction::ToggleDisplay);
                spawn_labeled_panel_button(row, SignAction::ClearDisplay);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        SignDisplayList,
        MaterialKind::ALL
            .into_iter()
            .map(|material| (material, SignAction::SetMaterial(material))),
        SignMaterialOption,
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click).add_systems(
        Update,
        (update_panel, update_dropdowns)
            .chain()
            .in_set(BlockPanelSystems),
    );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Sign,
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
    actions: Query<&SignAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Sign) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    if matches!(action, SignAction::EditText) {
        let current = world
            .world
            .sign_settings(pos)
            .text
            .unwrap_or_default();
        let spec = TextPromptProps {
            title: i18n.t("sign.prompt.text"),
            default_value: current,
            save_text: i18n.t("button.confirm"),
            cancel_text: i18n.t("button.cancel"),
        };
        ui.open_text_prompt_then(spec, move |result, world| {
            let TextPromptResult::Saved(requested) = result else {
                return;
            };
            let trimmed = requested.trim();
            let text = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.chars().take(64).collect::<String>())
            };
            if !world.resource::<WorldBlocks>().blocks.contains_key(&pos) {
                return;
            }
            let mut settings = world.resource::<WorldBlocks>().sign_settings(pos);
            settings.text = text;
            settings.display = None;
            let before = world
                .resource::<WorldBlocks>()
                .block_settings
                .get(&pos)
                .cloned();
            {
                let mut world_blocks = world.resource_mut::<WorldBlocks>();
                world_blocks.set_sign_settings(pos, settings);
            }
            let after = world
                .resource::<WorldBlocks>()
                .block_settings
                .get(&pos)
                .cloned();
            if let Some(mut history) = world.get_resource_mut::<EditHistory>() {
                history.record_settings(pos, before, after);
            }
            world.resource_mut::<SolutionState>().dirty = true;
        });
        return;
    }

    let mut settings = world.world.sign_settings(pos);
    let changed = match action {
        SignAction::EditText => unreachable!(),
        SignAction::ClearText => {
            settings.text = None;
            true
        }
        SignAction::ToggleDisplay => {
            open_dropdown.toggle(UiPanelId::Sign, DISPLAY_SLOT);
            return;
        }
        SignAction::SetMaterial(material) => {
            settings.display = Some(SignDisplay::Material(material));
            settings.text = None;
            open_dropdown.close();
            true
        }
        SignAction::ClearDisplay => {
            settings.display = None;
            true
        }
    };

    if changed {
        apply_block_settings_with_history(&mut edit_history, &mut world.world, pos, |blocks| {
            blocks.set_sign_settings(pos, settings);
        });
        solution_state.dirty = true;
        refresh_world_after_edit(&mut world, pos);
    }
}

fn update_panel(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut preview: Query<&mut Text, With<SignTextPreview>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    if ui_runtime.active_panel() != Some(UiPanelId::Sign) {
        return;
    }
    let settings = world.sign_settings(pos);
    let label = settings
        .text
        .as_deref()
        .filter(|text| !text.is_empty())
        .unwrap_or("-");
    for mut text in &mut preview {
        text.0 = label.to_string();
    }
}

fn update_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut display_slots: Query<(&SignDisplaySlot, &Children)>,
    mut material_options: Query<(&SignMaterialOption, &Children)>,
    mut material_icons: Query<&mut ImageNode>,
    mut lists: Query<(&SignDisplayList, &mut Node, &ComputedNode)>,
    triggers: Query<(&SignAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let active_pos = ui_runtime.active_block_pos();
    let panel = UiPanelId::Sign;

    if let Some(block_icons) = block_icons.as_deref() {
        let material = active_pos.and_then(|pos| match world.sign_settings(pos).display {
            Some(SignDisplay::Material(material)) => Some(material),
            _ => None,
        });
        for (_, children) in &mut display_slots {
            update_material_icon(children, material, block_icons, &mut material_icons);
        }
        for (option, children) in &mut material_options {
            update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);

    for (_, mut style, list_node) in &mut lists {
        let open = open_dropdown.is_open(panel, DISPLAY_SLOT);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == SignAction::ToggleDisplay && !node.is_empty()).then_some((node, transform))
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
