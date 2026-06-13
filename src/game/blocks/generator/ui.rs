use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::GeneratorBlock;

use crate::game::block_editing::widgets::{
    position_dropdown_from_trigger, spawn_labeled_panel_button, spawn_material_icon_list,
    spawn_material_icon_toggle, update_material_icon,
};
use crate::game::block_editing::world_refresh::refresh_world_after_edit;
use crate::game::block_editing::{BlockEditContext, OpenBlockPanelDropdown};
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::MaterialKind;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::UiMainThread;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::components::{
    default_button_size, localized_text, spawn_panel as spawn_ui_panel, text, transparent_node,
    PanelOptions,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::types::{UiActionLabel, UiPanelBinding};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;

const MATERIAL_SLOT: u8 = 0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorAction {
    PeriodDown,
    PeriodUp,
    ToggleMaterial,
    SetMaterial(MaterialKind),
}

#[derive(Component, Clone, Copy)]
struct GeneratorPeriodText;

#[derive(Component, Clone, Copy)]
struct GeneratorMaterialSlot;

#[derive(Component, Clone, Copy)]
struct GeneratorMaterialList;

#[derive(Component, Clone, Copy)]
struct GeneratorMaterialOption(MaterialKind);

impl UiActionLabel for GeneratorAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::PeriodDown => "button.period_down",
            Self::PeriodUp => "button.period_up",
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
            spawn_row(panel, "panel.period", |row| {
                spawn_labeled_panel_button(row, GeneratorAction::PeriodDown);
                row.spawn((
                    text("", 18.0, Color::WHITE),
                    GeneratorPeriodText,
                ));
                spawn_labeled_panel_button(row, GeneratorAction::PeriodUp);
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
        MaterialKind::ALL.into_iter().map(|material| {
            (
                material,
                GeneratorAction::SetMaterial(material),
            )
        }),
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

fn on_click(
    mut click: On<Pointer<Click>>,
    ui_host: Res<UiHost>,
    ui_runtime: Res<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut solution_state: ResMut<SolutionState>,
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
    );
}

fn dispatch_action(
    action: GeneratorAction,
    pos: IVec3,
    world: &mut PlayingWorldParams,
    solution_state: &mut SolutionState,
    open_dropdown: &mut OpenBlockPanelDropdown,
) {
    let mut ctx = BlockEditContext::new(pos, &mut world.world, solution_state, open_dropdown);
    let mut settings = ctx.world.generator_settings(pos);

    let changed = match action {
        GeneratorAction::PeriodDown => {
            settings.period = settings.period.saturating_sub(1).max(1);
            true
        }
        GeneratorAction::PeriodUp => {
            settings.period = (settings.period + 1).min(120);
            true
        }
        GeneratorAction::ToggleMaterial => {
            ctx.toggle_dropdown(UiPanelId::Generator, MATERIAL_SLOT);
            return;
        }
        GeneratorAction::SetMaterial(material) => {
            settings.material = material;
            ctx.close_dropdown();
            true
        }
    };

    if changed {
        ctx.world.set_generator_settings(pos, settings);
        ctx.mark_dirty();
        refresh_world_after_edit(world, pos);
    }
}

fn update_panel(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut period_text: Query<&mut Text, With<GeneratorPeriodText>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    if ui_runtime.active_panel() != Some(UiPanelId::Generator) {
        return;
    }
    let settings = world.generator_settings(pos);
    for mut text in &mut period_text {
        text.0 = settings.period.to_string();
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
    mut lists: Query<(
        &GeneratorMaterialList,
        &mut Node,
        &ComputedNode,
    )>,
    triggers: Query<
        (
            &GeneratorAction,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<Button>,
    >,
) {
    let active_pos = ui_runtime.active_block_pos();
    let panel = UiPanelId::Generator;

    if let Some(block_icons) = block_icons.as_deref() {
        let material = active_pos.map(|pos| world.generator_settings(pos).material);
        for (slot, children) in &mut material_slots {
            let _ = slot;
            update_material_icon(children, material, block_icons, &mut material_icons);
        }
        for (option, children) in &mut material_options {
            update_material_icon(
                children,
                Some(option.0),
                block_icons,
                &mut material_icons,
            );
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);

    for (list, mut style, list_node) in &mut lists {
        let _ = list;
        let open = open_dropdown.is_open(panel, MATERIAL_SLOT);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        let trigger = triggers.iter().find_map(|(action, node, transform)| {
            (*action == GeneratorAction::ToggleMaterial && !node.is_empty())
                .then_some((node, transform))
        });
        if let Some((trigger_node, transform)) = trigger {
            if let Some((left, top)) = position_dropdown_from_trigger(
                trigger_node,
                transform,
                list_node,
                viewport,
            ) {
                style.left = Val::Px(left);
                style.top = Val::Px(top);
            }
        }
    }
}
