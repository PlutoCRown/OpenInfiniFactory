use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::ConverterBlock;

use crate::game::block_editing::widgets::{
    position_dropdown_from_trigger, spawn_material_icon_list, spawn_material_icon_toggle,
    ui_transform_scale, update_material_icon,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::block_editing::{BlockEditContext, OpenBlockPanelDropdown};
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::MaterialKind;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::UiMainThread;
use crate::game::ui::components::{
    default_button_size, localized_text, spawn_panel as spawn_ui_panel, transparent_node,
    PanelOptions,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::{ConverterMode, WorldBlocks};
use crate::game::world::rendering::BlockIconAssets;

const INPUT_SLOT: u8 = 0;
const OUTPUT_SLOT: u8 = 1;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConverterAction {
    ToggleInput,
    ToggleOutput,
    SetInput(MaterialKind),
    SetOutput(MaterialKind),
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
struct ConverterMaterialOption(MaterialKind);

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
        MaterialKind::ALL
            .into_iter()
            .map(|m| (m, ConverterAction::SetInput(m))),
        ConverterMaterialOption,
    );
    spawn_material_icon_list(
        root,
        ConverterOutputList,
        MaterialKind::ALL
            .into_iter()
            .map(|m| (m, ConverterAction::SetOutput(m))),
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
    mut solution_state: ResMut<SolutionState>,
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

    let mut ctx = BlockEditContext::new(
        pos,
        &mut world.world,
        &mut solution_state,
        &mut open_dropdown,
    );
    let mut settings = ctx.world.converter_settings(pos);
    let mut changed = false;

    match action {
        ConverterAction::ToggleInput => {
            ctx.toggle_dropdown(UiPanelId::Converter, INPUT_SLOT);
            return;
        }
        ConverterAction::ToggleOutput => {
            ctx.toggle_dropdown(UiPanelId::Converter, OUTPUT_SLOT);
            return;
        }
        ConverterAction::SetInput(material) => {
            settings.input = material;
            settings.mode = ConverterMode::SpecificInput;
            ctx.close_dropdown();
            changed = true;
        }
        ConverterAction::SetOutput(material) => {
            settings.output = material;
            ctx.close_dropdown();
            changed = true;
        }
    }

    if changed {
        ctx.world.set_converter_settings(pos, settings);
        ctx.mark_dirty();
        refresh_world_after_edit(&mut world);
    }
}

fn show_input_row(ui_runtime: Res<UiRuntime>, mut rows: Query<&mut Node, With<ConverterInputRow>>) {
    if ui_runtime.active_panel() != Some(UiPanelId::Converter) {
        return;
    }
    for mut style in &mut rows {
        style.display = Display::Flex;
    }
}

fn update_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
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
    let active_pos = ui_runtime.active_block_pos();
    let panel = UiPanelId::Converter;

    if let Some(block_icons) = block_icons.as_deref() {
        if let Some(pos) = active_pos {
            let settings = world.converter_settings(pos);
            for (_, children) in &mut material_slots {
                update_material_icon(
                    children,
                    Some(settings.input),
                    block_icons,
                    &mut material_icons,
                );
            }
            for (_, children) in &mut output_slots {
                update_material_icon(
                    children,
                    Some(settings.output),
                    block_icons,
                    &mut material_icons,
                );
            }
        }
        for (option, children) in &mut material_options {
            update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);
    let scale = ui_transform_scale(window, ui_scale.0);

    update_material_list(
        &open_dropdown,
        panel,
        INPUT_SLOT,
        ConverterAction::ToggleInput,
        &mut list_queries.p0(),
        &triggers,
        viewport,
        scale,
    );
    update_material_list(
        &open_dropdown,
        panel,
        OUTPUT_SLOT,
        ConverterAction::ToggleOutput,
        &mut list_queries.p1(),
        &triggers,
        viewport,
        scale,
    );
}

fn update_material_list<L>(
    open_dropdown: &OpenBlockPanelDropdown,
    panel: UiPanelId,
    slot: u8,
    toggle: ConverterAction,
    lists: &mut Query<(&L, &mut Node, &ComputedNode)>,
    triggers: &Query<(&ConverterAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    viewport: Vec2,
    scale: f32,
) where
    L: Component,
{
    for (_, mut style, list_node) in lists.iter_mut() {
        let open = open_dropdown.is_open(panel, slot);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == toggle && !node.is_empty()).then_some((node, transform))
        });
        if let Some((trigger_node, transform)) = trigger {
            if let Some((left, top)) = position_dropdown_from_trigger(
                trigger_node,
                transform,
                list_node.size(),
                viewport,
                scale,
            ) {
                style.left = Val::Px(left);
                style.top = Val::Px(top);
            }
        }
    }
}
