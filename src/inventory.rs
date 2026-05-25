use bevy::prelude::*;

use crate::blocks::{BlockKind, PLACEABLE_BLOCKS};
use crate::state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};

pub const HOTBAR_SLOTS: usize = 9;
const BACKPACK_SLOTS: usize = 27;

#[derive(Component)]
pub struct HotbarText;

#[derive(Component)]
pub struct BackpackPanel;

#[derive(Component)]
pub struct PausePanel;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct FovText;

#[derive(Component)]
pub struct SimulationText;

#[derive(Component, Clone, Copy)]
pub enum PauseAction {
    Resume,
    ToggleBuilderMode,
    FovDown,
    FovUp,
    Quit,
}

#[derive(Component, Clone, Copy)]
pub enum SimulationAction {
    ToggleRun,
    Faster,
    Rollback,
}

#[derive(Component)]
pub(crate) struct SlotLabel;

#[derive(Component)]
pub(crate) struct CarriedLabel;

#[derive(Component, Clone, Copy)]
pub(crate) struct InventorySlot {
    area: SlotArea,
    index: usize,
}

#[derive(Resource)]
pub struct InventoryItems {
    pub hotbar: [Option<BlockKind>; HOTBAR_SLOTS],
    backpack: [Option<BlockKind>; BACKPACK_SLOTS],
}

impl Default for InventoryItems {
    fn default() -> Self {
        let mut hotbar = [None; HOTBAR_SLOTS];
        for (index, kind) in PLACEABLE_BLOCKS.iter().enumerate() {
            hotbar[index] = Some(*kind);
        }

        let mut backpack = [None; BACKPACK_SLOTS];
        for index in 0..BACKPACK_SLOTS {
            backpack[index] = Some(PLACEABLE_BLOCKS[index % PLACEABLE_BLOCKS.len()]);
        }

        Self { hotbar, backpack }
    }
}

#[derive(Resource)]
pub struct CarriedItem(Option<BlockKind>);

impl Default for CarriedItem {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SlotArea {
    Hotbar,
    Backpack,
}

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "+",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        ..default()
                    },
                    ..default()
                },
                Crosshair,
            ));

            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(18.0),
                        bottom: Val::Px(92.0),
                        ..default()
                    },
                    ..default()
                },
                HotbarText,
            ));

            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(0.88, 0.96, 1.0),
                            ..default()
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(18.0),
                        top: Val::Px(118.0),
                        ..default()
                    },
                    ..default()
                },
                SimulationText,
            ));

            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(260.0),
                    height: Val::Px(38.0),
                    position_type: PositionType::Absolute,
                    right: Val::Px(18.0),
                    top: Val::Px(182.0),
                    display: Display::Flex,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|bar| {
                spawn_sim_button(bar, "Play/Pause", SimulationAction::ToggleRun);
                spawn_sim_button(bar, "Speed", SimulationAction::Faster);
                spawn_sim_button(bar, "Rollback", SimulationAction::Rollback);
            });

            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(540.0),
                    height: Val::Px(58.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(22.0),
                    margin: UiRect {
                        left: Val::Px(-270.0),
                        ..default()
                    },
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
                background_color: Color::srgba(0.04, 0.04, 0.04, 0.38).into(),
                ..default()
            })
            .with_children(|bar| {
                for index in 0..HOTBAR_SLOTS {
                    spawn_slot(bar, SlotArea::Hotbar, index);
                }
            });

            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(540.0),
                        height: Val::Px(350.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-270.0),
                            top: Val::Px(-175.0),
                            ..default()
                        },
                        padding: UiRect::all(Val::Px(18.0)),
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    background_color: Color::srgba(0.12, 0.12, 0.13, 0.94).into(),
                    ..default()
                },
                BackpackPanel,
            ))
            .with_children(|panel| {
                panel.spawn(TextBundle::from_section(
                    "Inventory",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.94, 0.94, 0.92),
                        ..default()
                    },
                ));

                panel
                    .spawn(NodeBundle {
                        style: Style {
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
                            grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                            row_gap: Val::Px(4.0),
                            column_gap: Val::Px(4.0),
                            width: Val::Px(504.0),
                            height: Val::Px(164.0),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    })
                    .with_children(|grid| {
                        for index in 0..BACKPACK_SLOTS {
                            spawn_slot(grid, SlotArea::Backpack, index);
                        }
                    });

                panel.spawn(TextBundle::from_section(
                    "Click a slot to pick up or swap. Number keys select the hotbar.",
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.78, 0.78, 0.76),
                        ..default()
                    },
                ));
            });

            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(380.0),
                        height: Val::Px(270.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-190.0),
                            top: Val::Px(-135.0),
                            ..default()
                        },
                        padding: UiRect::all(Val::Px(20.0)),
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    background_color: Color::srgba(0.08, 0.09, 0.10, 0.94).into(),
                    ..default()
                },
                PausePanel,
            ))
            .with_children(|panel| {
                panel.spawn(TextBundle::from_section(
                    "Paused",
                    TextStyle {
                        font_size: 30.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
                spawn_pause_button(panel, "Resume", PauseAction::Resume);
                spawn_pause_button(
                    panel,
                    "Toggle Edit/Play Mode",
                    PauseAction::ToggleBuilderMode,
                );

                panel
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(42.0),
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_pause_button(row, "FOV -", PauseAction::FovDown);
                        row.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            FovText,
                        ));
                        spawn_pause_button(row, "FOV +", PauseAction::FovUp);
                    });

                spawn_pause_button(panel, "Quit Game", PauseAction::Quit);
            });

            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(18.0),
                            top: Val::Px(18.0),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },
                CarriedLabel,
            ));
        });
}

pub fn inventory_slot_clicks(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>),
    >,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mode: Res<GameMode>,
) {
    if *mode != GameMode::Inventory {
        return;
    }

    for (interaction, slot) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let slot_item = match slot.area {
            SlotArea::Hotbar => &mut inventory.hotbar[slot.index],
            SlotArea::Backpack => &mut inventory.backpack[slot.index],
        };
        std::mem::swap(slot_item, &mut carried.0);
    }
}

pub fn update_ui(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    mode: Res<GameMode>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    settings: Res<GameSettings>,
    carried: Res<CarriedItem>,
    mut hotbar: Query<&mut Text, (With<HotbarText>, Without<SlotLabel>, Without<CarriedLabel>)>,
    mut panels: Query<&mut Style, With<BackpackPanel>>,
    mut pause_panels: Query<&mut Style, (With<PausePanel>, Without<BackpackPanel>)>,
    mut crosshair: Query<&mut Visibility, With<Crosshair>>,
    mut slot_query: Query<
        (
            &InventorySlot,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut Text, (With<SlotLabel>, Without<HotbarText>, Without<CarriedLabel>)>,
    mut carried_label: Query<
        &mut Text,
        (
            With<CarriedLabel>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<FovText>,
        ),
    >,
    mut fov_text: Query<
        &mut Text,
        (
            With<FovText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
        ),
    >,
    mut simulation_text: Query<
        &mut Text,
        (
            With<SimulationText>,
            Without<SlotLabel>,
            Without<HotbarText>,
            Without<CarriedLabel>,
            Without<FovText>,
        ),
    >,
) {
    if let Ok(mut text) = hotbar.get_single_mut() {
        let selected = inventory.hotbar[placement.selected]
            .map(BlockKind::name)
            .unwrap_or("Empty");
        text.sections[0].value = format!(
            "Mode: {}   Selected: {}   Facing: {}   E: Inventory   ESC: Pause",
            builder_mode_name(*builder_mode),
            selected,
            placement.facing.name()
        );
    }

    for mut style in &mut panels {
        style.display = if *mode == GameMode::Inventory {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut pause_panels {
        style.display = if *mode == GameMode::Paused {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut visibility in &mut crosshair {
        *visibility = if *mode == GameMode::Playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (slot, children, mut background, mut border) in &mut slot_query {
        let item = match slot.area {
            SlotArea::Hotbar => inventory.hotbar[slot.index],
            SlotArea::Backpack => inventory.backpack[slot.index],
        };

        let selected_hotbar = slot.area == SlotArea::Hotbar && slot.index == placement.selected;
        *background = item
            .map(slot_color)
            .unwrap_or(Color::srgba(0.16, 0.16, 0.17, 0.92))
            .into();
        *border = if selected_hotbar {
            Color::srgb(1.0, 1.0, 1.0).into()
        } else {
            Color::srgb(0.22, 0.22, 0.22).into()
        };

        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(*child) {
                text.sections[0].value = item
                    .map(|kind| short_item_name(kind).to_string())
                    .unwrap_or_default();
            }
        }
    }

    if let Ok(mut text) = carried_label.get_single_mut() {
        text.sections[0].value = carried
            .0
            .map(|kind| format!("Holding: {}", kind.name()))
            .unwrap_or_default();
    }

    if let Ok(mut text) = fov_text.get_single_mut() {
        text.sections[0].value = format!("FOV {:.0}", settings.fov_degrees);
    }

    if let Ok(mut text) = simulation_text.get_single_mut() {
        text.sections[0].value = format!(
            "Mode: {}\nTurns: {}\nSim: {} x{:.0}",
            builder_mode_name(*builder_mode),
            simulation.turn,
            if simulation.running {
                "Playing"
            } else {
                "Paused"
            },
            simulation.speed
        );
    }
}

fn spawn_slot(parent: &mut ChildBuilder, area: SlotArea, index: usize) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(54.0),
                    height: Val::Px(54.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                border_color: Color::srgb(0.22, 0.22, 0.22).into(),
                background_color: Color::srgba(0.28, 0.28, 0.30, 0.92).into(),
                ..default()
            },
            InventorySlot { area, index },
        ))
        .with_children(|slot| {
            slot.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_justify(JustifyText::Center),
                    style: Style {
                        margin: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    ..default()
                },
                SlotLabel,
            ));
        });
}

fn spawn_pause_button(parent: &mut ChildBuilder, label: &'static str, action: PauseAction) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_width: Val::Px(92.0),
                    height: Val::Px(38.0),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                border_color: Color::srgb(0.38, 0.39, 0.40).into(),
                background_color: Color::srgba(0.22, 0.24, 0.26, 0.96).into(),
                ..default()
            },
            action,
        ))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn spawn_sim_button(parent: &mut ChildBuilder, label: &'static str, action: SimulationAction) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(82.0),
                    height: Val::Px(34.0),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                border_color: Color::srgb(0.30, 0.36, 0.40).into(),
                background_color: Color::srgba(0.12, 0.18, 0.22, 0.84).into(),
                ..default()
            },
            action,
        ))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn builder_mode_name(mode: BuilderMode) -> &'static str {
    match mode {
        BuilderMode::Edit => "Edit",
        BuilderMode::Play => "Play",
    }
}

fn slot_color(kind: BlockKind) -> Color {
    match kind {
        BlockKind::Solid => Color::srgb(0.38, 0.39, 0.40),
        BlockKind::Glass => Color::srgb(0.42, 0.66, 0.76),
        BlockKind::Generator => Color::srgb(0.42, 0.20, 0.56),
        BlockKind::Welder => Color::srgb(0.62, 0.12, 0.12),
        BlockKind::Conveyor => Color::srgb(0.08, 0.20, 0.26),
        BlockKind::Piston => Color::srgb(0.66, 0.43, 0.20),
        BlockKind::Goal => Color::srgb(0.24, 0.56, 0.30),
        BlockKind::Material => Color::srgb(0.74, 0.74, 0.78),
        BlockKind::WeldPoint => Color::srgb(0.86, 0.16, 0.12),
    }
}

fn short_item_name(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Solid => "Solid",
        BlockKind::Glass => "Glass",
        BlockKind::Generator => "Gen",
        BlockKind::Welder => "Weld",
        BlockKind::Conveyor => "Belt",
        BlockKind::Piston => "Piston",
        BlockKind::Goal => "Goal",
        BlockKind::Material => "Mat",
        BlockKind::WeldPoint => "Point",
    }
}
