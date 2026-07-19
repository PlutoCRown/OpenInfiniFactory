use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::GeneratorBlock;

use crate::game::edit_history::EditHistory;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    click_material_slot, spawn_labeled_panel_button, spawn_material_icon_list,
    spawn_material_icon_toggle, sync_dropdown_overlay, update_material_icon,
};
use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{MaterialBlockId, material_catalog};
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::{
    PanelOptions, default_button_size, localized_text, spawn_panel as spawn_ui_panel, text,
    transparent_node,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{CarriedItem, UiActionLabel, UiPanelBinding};
use crate::game::world::grid::{GeneratorMode, WorldBlocks};
use crate::game::world::rendering::BlockIconAssets;

const MATERIAL_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorAction {
    ToggleMode,
    PeriodDown,
    PeriodUp,
    OffsetDown,
    OffsetUp,
    AcceptorPrev,
    AcceptorNext,
    ToggleMaterial,
    SetMaterial(MaterialBlockId),
}

#[derive(Component, Clone, Copy)]
struct GeneratorModeText;

#[derive(Component, Clone, Copy)]
struct GeneratorPeriodText;

#[derive(Component, Clone, Copy)]
struct GeneratorOffsetText;

#[derive(Component, Clone, Copy)]
struct GeneratorAcceptorText;

#[derive(Component, Clone, Copy)]
struct GeneratorPeriodRow;

#[derive(Component, Clone, Copy)]
struct GeneratorOffsetRow;

#[derive(Component, Clone, Copy)]
struct GeneratorAcceptorRow;

#[derive(Component, Clone, Copy)]
struct GeneratorMaterialSlot;

#[derive(Component, Clone, Copy)]
struct GeneratorMaterialList;

#[derive(Component, Clone, Copy)]
struct GeneratorMaterialOption(MaterialBlockId);

impl UiActionLabel for GeneratorAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleMode => "button.generator_mode",
            Self::PeriodDown | Self::OffsetDown | Self::AcceptorPrev => "button.period_down",
            Self::PeriodUp | Self::OffsetUp | Self::AcceptorNext => "button.period_up",
            Self::ToggleMaterial | Self::SetMaterial(_) => "button.material_next",
        }
    }
}

impl BlockUi for GeneratorBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Generator)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_ui_panel(
        root,
        PanelOptions::new(430.0, "generator.title").closable(),
        UiPanelBinding(UiPanelId::Generator),
        |panel| {
            spawn_row(panel, "panel.mode", |row| {
                spawn_labeled_panel_button(row, GeneratorAction::ToggleMode);
                row.spawn((text("", 18.0, Color::WHITE), GeneratorModeText));
            });
            spawn_tagged_row(panel, "panel.period", GeneratorPeriodRow, |row| {
                spawn_labeled_panel_button(row, GeneratorAction::PeriodDown);
                row.spawn((text("", 18.0, Color::WHITE), GeneratorPeriodText));
                spawn_labeled_panel_button(row, GeneratorAction::PeriodUp);
            });
            spawn_tagged_row(panel, "panel.offset", GeneratorOffsetRow, |row| {
                spawn_labeled_panel_button(row, GeneratorAction::OffsetDown);
                row.spawn((text("", 18.0, Color::WHITE), GeneratorOffsetText));
                spawn_labeled_panel_button(row, GeneratorAction::OffsetUp);
            });
            spawn_tagged_row(panel, "panel.acceptor", GeneratorAcceptorRow, |row| {
                spawn_labeled_panel_button(row, GeneratorAction::AcceptorPrev);
                row.spawn((text("", 18.0, Color::WHITE), GeneratorAcceptorText));
                spawn_labeled_panel_button(row, GeneratorAction::AcceptorNext);
            });
            spawn_row(panel, "panel.material", |row| {
                spawn_material_icon_toggle(
                    row,
                    GeneratorMaterialSlot,
                    GeneratorAction::ToggleMaterial,
                );
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        GeneratorMaterialList,
        material_catalog()
            .iter()
            .map(|(id, _)| (id, GeneratorAction::SetMaterial(id))),
        GeneratorMaterialOption,
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
        panel: UiPanelId::Generator,
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

fn spawn_tagged_row<T: Component>(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    tag: T,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel
        .spawn((
            transparent_node(Node {
                width: Val::Percent(100.0),
                height: Val::Px(default_button_size(40.0)),
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            }),
            tag,
        ))
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
    mut carried: ResMut<CarriedItem>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&GeneratorAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Generator) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    dispatch_action(
        action,
        pos,
        &mut world,
        &mut solution_state,
        &mut open_dropdown,
        &mut carried,
        &mut edit_history,
    );
}

fn dispatch_action(
    action: GeneratorAction,
    pos: IVec3,
    world: &mut PlayingWorldParams,
    solution_state: &mut SolutionState,
    open_dropdown: &mut OpenBlockPanelDropdown,
    carried: &mut CarriedItem,
    edit_history: &mut EditHistory,
) {
    let mut settings = world.world.generator_settings(pos);
    let acceptor_anchors: Vec<IVec3> = world
        .world
        .acceptor_structures
        .iter()
        .filter_map(|structure| structure.positions.first().copied())
        .collect();

    let changed = match action {
        GeneratorAction::ToggleMode => {
            settings.mode = match settings.mode {
                GeneratorMode::Period { .. } => GeneratorMode::Link {
                    anchor: acceptor_anchors.first().copied(),
                },
                GeneratorMode::Link { .. } => GeneratorMode::Period {
                    period: crate::game::blocks::DEFAULT_GENERATOR_PERIOD,
                    offset: 0,
                },
            };
            true
        }
        GeneratorAction::PeriodDown => {
            if let GeneratorMode::Period { period, offset } = &mut settings.mode {
                *period = period.saturating_sub(1).max(1);
                *offset %= *period;
            }
            true
        }
        GeneratorAction::PeriodUp => {
            if let GeneratorMode::Period { period, offset } = &mut settings.mode {
                *period = (*period + 1).min(120);
                *offset %= *period;
            }
            true
        }
        GeneratorAction::OffsetDown => {
            if let GeneratorMode::Period { period, offset } = &mut settings.mode {
                let period = (*period).max(1);
                *offset = offset.saturating_sub(1) % period;
            }
            true
        }
        GeneratorAction::OffsetUp => {
            if let GeneratorMode::Period { period, offset } = &mut settings.mode {
                let period = (*period).max(1);
                *offset = (*offset + 1) % period;
            }
            true
        }
        GeneratorAction::AcceptorPrev => {
            if let GeneratorMode::Link { anchor } = &mut settings.mode {
                *anchor = cycle_anchor(*anchor, &acceptor_anchors, false);
            }
            true
        }
        GeneratorAction::AcceptorNext => {
            if let GeneratorMode::Link { anchor } = &mut settings.mode {
                *anchor = cycle_anchor(*anchor, &acceptor_anchors, true);
            }
            true
        }
        GeneratorAction::ToggleMaterial => {
            if let Some(material) =
                click_material_slot(UiPanelId::Generator, MATERIAL_SLOT, carried, open_dropdown)
            {
                settings.material = material;
                true
            } else {
                return;
            }
        }
        GeneratorAction::SetMaterial(material) => {
            settings.material = material;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_edit(edit_history, world, pos, |blocks| {
            blocks.set_generator_settings(pos, settings);
        });
        solution_state.dirty = true;
    }
}

fn cycle_anchor(current: Option<IVec3>, anchors: &[IVec3], forward: bool) -> Option<IVec3> {
    if anchors.is_empty() {
        return None;
    }
    let Some(current) = current else {
        return Some(anchors[0]);
    };
    let Some(index) = anchors.iter().position(|pos| *pos == current) else {
        return Some(anchors[0]);
    };
    if forward {
        Some(anchors[(index + 1) % anchors.len()])
    } else {
        Some(anchors[(index + anchors.len() - 1) % anchors.len()])
    }
}

fn update_panel(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut mode_text: Query<&mut Text, With<GeneratorModeText>>,
    mut period_text: Query<&mut Text, (With<GeneratorPeriodText>, Without<GeneratorModeText>)>,
    mut offset_text: Query<
        &mut Text,
        (
            With<GeneratorOffsetText>,
            Without<GeneratorModeText>,
            Without<GeneratorPeriodText>,
        ),
    >,
    mut acceptor_text: Query<
        &mut Text,
        (
            With<GeneratorAcceptorText>,
            Without<GeneratorModeText>,
            Without<GeneratorPeriodText>,
            Without<GeneratorOffsetText>,
        ),
    >,
    mut period_rows: Query<&mut Node, With<GeneratorPeriodRow>>,
    mut offset_rows: Query<&mut Node, (With<GeneratorOffsetRow>, Without<GeneratorPeriodRow>)>,
    mut acceptor_rows: Query<
        &mut Node,
        (
            With<GeneratorAcceptorRow>,
            Without<GeneratorPeriodRow>,
            Without<GeneratorOffsetRow>,
        ),
    >,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    if ui_runtime.active_panel() != Some(UiPanelId::Generator) {
        return;
    }
    let settings = world.generator_settings(pos);
    let is_period = matches!(settings.mode, GeneratorMode::Period { .. });
    for mut node in &mut period_rows {
        node.display = if is_period {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut node in &mut offset_rows {
        node.display = if is_period {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut node in &mut acceptor_rows {
        node.display = if is_period {
            Display::None
        } else {
            Display::Flex
        };
    }

    match settings.mode {
        GeneratorMode::Period { period, offset } => {
            for mut text in &mut mode_text {
                text.0 = i18n.t("generator.mode_period");
            }
            for mut text in &mut period_text {
                text.0 = period.to_string();
            }
            for mut text in &mut offset_text {
                text.0 = offset.to_string();
            }
        }
        GeneratorMode::Link { anchor } => {
            for mut text in &mut mode_text {
                text.0 = i18n.t("generator.mode_link");
            }
            for mut text in &mut acceptor_text {
                text.0 = match anchor.and_then(|pos| world.acceptor_id_at(pos)) {
                    None => "-".to_string(),
                    Some(id) => format!("#{}", id.0),
                };
            }
        }
    }
}

fn update_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut material_slots: Query<(&GeneratorMaterialSlot, &Children)>,
    mut material_options: Query<(&GeneratorMaterialOption, &Children)>,
    mut material_icons: Query<&mut ImageNode>,
    mut lists: Query<(&GeneratorMaterialList, &mut Node, &ComputedNode)>,
    triggers: Query<(&GeneratorAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Generator;
    let panel_active = ui_runtime.active_panel() == Some(panel);
    let open = panel_active && open_dropdown.is_open(panel, MATERIAL_SLOT);

    let window = windows.single().ok();
    let viewport = window
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);
    for (_, mut style, list_node) in &mut lists {
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == GeneratorAction::ToggleMaterial && !node.is_empty())
                .then_some((node, transform))
        });
        sync_dropdown_overlay(open, &mut style, list_node, trigger, viewport);
    }

    if !panel_active {
        return;
    }

    // 不缓存「已填充」：关面板时本系统被 run_if 跳过，Local 清不掉，二次打开会跳过刷新
    let Some(icons) = block_icons.as_ref() else {
        return;
    };
    let block_icons = icons.as_ref();
    for (option, children) in &mut material_options {
        update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
    }

    let material = ui_runtime
        .active_block_pos()
        .map(|pos| world.generator_settings(pos).material);
    for (_, children) in &mut material_slots {
        update_material_icon(children, material, block_icons, &mut material_icons);
    }
}
