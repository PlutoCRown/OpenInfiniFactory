use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::ConverterBlock;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    click_material_slot, spawn_material_icon_list, spawn_material_icon_toggle,
    sync_dropdown_overlay, update_material_icon,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{MaterialBlockId, material_catalog};
use crate::game::edit_history::{EditHistory, apply_block_settings_with_history};
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::UiMainThread;
use crate::game::ui::components::{
    PanelOptions, default_button_size, localized_text, spawn_panel as spawn_ui_panel,
    transparent_node,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{CarriedItem, UiActionLabel, UiPanelBinding};
use crate::game::world::grid::{ConverterMode, WorldBlocks};
use crate::game::world::rendering::BlockIconAssets;

const INPUT_SLOT: u8 = 0;
const OUTPUT_SLOT: u8 = 1;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConverterAction {
    ToggleInput,
    ToggleOutput,
    SetInput(MaterialBlockId),
    SetOutput(MaterialBlockId),
}

#[derive(Component, Clone, Copy)]
struct ConverterInputRow;

#[derive(Component, Clone, Copy)]
struct ConverterInputSlot;

#[derive(Component, Clone, Copy)]
struct ConverterOutputSlot;

#[derive(Component, Clone, Copy)]
struct ConverterInputList;

#[derive(Component, Clone, Copy)]
struct ConverterOutputList;

#[derive(Component, Clone, Copy)]
struct ConverterMaterialOption(MaterialBlockId);

impl UiActionLabel for ConverterAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleInput | Self::SetInput(_) => "button.input_material",
            Self::ToggleOutput | Self::SetOutput(_) => "button.output_material",
        }
    }
}

impl BlockUi for ConverterBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Converter)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_ui_panel(
        root,
        PanelOptions::new(460.0, "converter.title").closable(),
        UiPanelBinding(UiPanelId::Converter),
        |panel| {
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
                    ConverterInputRow,
                ))
                .with_children(|row| {
                    row.spawn((
                        localized_text("panel.input", 16.0, Color::srgb(0.86, 0.88, 0.86)),
                        Node {
                            width: Val::Px(110.0),
                            ..default()
                        },
                    ));
                    spawn_material_icon_toggle(
                        row,
                        ConverterInputSlot,
                        ConverterAction::ToggleInput,
                    );
                });
            spawn_row(panel, "panel.output", |row| {
                spawn_material_icon_toggle(row, ConverterOutputSlot, ConverterAction::ToggleOutput);
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        ConverterInputList,
        material_catalog()
            .iter()
            .map(|(id, _)| (id, ConverterAction::SetInput(id))),
        ConverterMaterialOption,
    );
    spawn_material_icon_list(
        root,
        ConverterOutputList,
        material_catalog()
            .iter()
            .map(|(id, _)| (id, ConverterAction::SetOutput(id))),
        ConverterMaterialOption,
    );
}

pub fn register(app: &mut App) {
    app.add_observer(on_click).add_systems(
        Update,
        (show_input_row, update_dropdowns)
            .chain()
            .in_set(BlockPanelSystems),
    );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Converter,
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
    mut carried: ResMut<CarriedItem>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&ConverterAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Converter) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let mut settings = world.world.converter_settings(pos);

    let changed = match action {
        ConverterAction::ToggleInput => {
            if let Some(material) = click_material_slot(
                UiPanelId::Converter,
                INPUT_SLOT,
                &mut carried,
                &mut open_dropdown,
            ) {
                settings.input = material;
                settings.mode = ConverterMode::SpecificInput;
                true
            } else {
                return;
            }
        }
        ConverterAction::ToggleOutput => {
            if let Some(material) = click_material_slot(
                UiPanelId::Converter,
                OUTPUT_SLOT,
                &mut carried,
                &mut open_dropdown,
            ) {
                settings.output = material;
                true
            } else {
                return;
            }
        }
        ConverterAction::SetInput(material) => {
            settings.input = material;
            settings.mode = ConverterMode::SpecificInput;
            open_dropdown.close();
            true
        }
        ConverterAction::SetOutput(material) => {
            settings.output = material;
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_with_history(&mut edit_history, &mut world.world, pos, |blocks| {
            blocks.set_converter_settings(pos, settings);
        });
        solution_state.dirty = true;
        refresh_world_after_edit(&mut world, pos);
    }
}

fn show_input_row(ui_runtime: Res<UiRuntime>, mut rows: Query<&mut Node, With<ConverterInputRow>>) {
    if ui_runtime.active_panel() != Some(UiPanelId::Converter) {
        return;
    }
    for mut style in &mut rows {
        if style.display != Display::Flex {
            style.display = Display::Flex;
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
    mut option_icons_filled: Local<bool>,
    mut last_slot_materials: Local<Option<(Option<MaterialBlockId>, Option<MaterialBlockId>)>>,
    mut material_slots: Query<(&ConverterInputSlot, &Children)>,
    mut output_slots: Query<(&ConverterOutputSlot, &Children)>,
    mut material_options: Query<(&ConverterMaterialOption, &Children)>,
    mut material_icons: Query<&mut ImageNode>,
    mut list_queries: ParamSet<(
        Query<(&ConverterInputList, &mut Node, &ComputedNode)>,
        Query<(&ConverterOutputList, &mut Node, &ComputedNode)>,
    )>,
    triggers: Query<(&ConverterAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Converter;
    let panel_active = ui_runtime.active_panel() == Some(panel);
    let input_open = panel_active && open_dropdown.is_open(panel, INPUT_SLOT);
    let output_open = panel_active && open_dropdown.is_open(panel, OUTPUT_SLOT);

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);

    update_material_list(
        input_open,
        ConverterAction::ToggleInput,
        &mut list_queries.p0(),
        &triggers,
        viewport,
    );
    update_material_list(
        output_open,
        ConverterAction::ToggleOutput,
        &mut list_queries.p1(),
        &triggers,
        viewport,
    );

    let Some(icons) = block_icons.as_ref() else {
        return;
    };
    let icons_changed = icons.is_changed();
    let block_icons = icons.as_ref();
    if !*option_icons_filled || icons_changed {
        for (option, children) in &mut material_options {
            update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
        }
        *option_icons_filled = true;
    }

    if !panel_active {
        *last_slot_materials = None;
        return;
    }
    let slot_materials = ui_runtime
        .active_block_pos()
        .map(|pos| {
            let settings = world.converter_settings(pos);
            (Some(settings.input), Some(settings.output))
        })
        .unwrap_or((None, None));
    if last_slot_materials.as_ref() != Some(&slot_materials) || icons_changed {
        for (_, children) in &mut material_slots {
            update_material_icon(children, slot_materials.0, block_icons, &mut material_icons);
        }
        for (_, children) in &mut output_slots {
            update_material_icon(children, slot_materials.1, block_icons, &mut material_icons);
        }
        *last_slot_materials = Some(slot_materials);
    }
}

fn update_material_list<L>(
    open: bool,
    toggle: ConverterAction,
    lists: &mut Query<(&L, &mut Node, &ComputedNode)>,
    triggers: &Query<(&ConverterAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    viewport: Vec2,
) where
    L: Component,
{
    for (_, mut style, list_node) in lists.iter_mut() {
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == toggle && !node.is_empty()).then_some((node, transform))
        });
        sync_dropdown_overlay(open, &mut style, list_node, trigger, viewport);
    }
}
