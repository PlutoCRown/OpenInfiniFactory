use bevy::prelude::*;

use crate::blocks::{BlockKind, ALL_BLOCKS};
use crate::{GameMode, PlacementState};

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
        for (index, kind) in ALL_BLOCKS.iter().enumerate() {
            hotbar[index] = Some(*kind);
        }

        let mut backpack = [None; BACKPACK_SLOTS];
        for index in 0..BACKPACK_SLOTS {
            backpack[index] = Some(ALL_BLOCKS[index % ALL_BLOCKS.len()]);
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
                        width: Val::Px(340.0),
                        height: Val::Px(170.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-170.0),
                            top: Val::Px(-85.0),
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
                panel.spawn(TextBundle::from_section(
                    "Press ESC or left click to return to the game.",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.82, 0.84, 0.84),
                        ..default()
                    },
                ));
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
        (With<CarriedLabel>, Without<SlotLabel>, Without<HotbarText>),
    >,
) {
    if let Ok(mut text) = hotbar.get_single_mut() {
        let selected = inventory.hotbar[placement.selected]
            .map(BlockKind::name)
            .unwrap_or("Empty");
        text.sections[0].value = format!(
            "Selected: {}   Facing: {}   E: Inventory   ESC: Pause",
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

fn slot_color(kind: BlockKind) -> Color {
    match kind {
        BlockKind::Solid => Color::srgb(0.38, 0.39, 0.40),
        BlockKind::Conveyor => Color::srgb(0.08, 0.20, 0.26),
        BlockKind::Piston => Color::srgb(0.66, 0.43, 0.20),
        BlockKind::Glass => Color::srgb(0.42, 0.66, 0.76),
        BlockKind::Goal => Color::srgb(0.24, 0.56, 0.30),
    }
}

fn short_item_name(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Solid => "Solid",
        BlockKind::Conveyor => "Belt",
        BlockKind::Piston => "Piston",
        BlockKind::Glass => "Glass",
        BlockKind::Goal => "Goal",
    }
}
