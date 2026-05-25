use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SAVE_PATH: &str = "saves/world.ron";
const REACH: f32 = 8.0;
const BLOCK_SIZE: f32 = 1.0;
const EYE_HEIGHT: f32 = 1.7;
const PLAYER_SPEED: f32 = 5.5;
const JUMP_SPEED: f32 = 6.5;
const GRAVITY: f32 = 18.0;
const HOTBAR_SLOTS: usize = 9;
const BACKPACK_SLOTS: usize = 27;
const ALL_BLOCKS: [BlockKind; 5] = [
    BlockKind::Solid,
    BlockKind::Conveyor,
    BlockKind::Piston,
    BlockKind::Glass,
    BlockKind::Goal,
];

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
        .insert_resource(WorldBlocks::default())
        .insert_resource(PlacementState::default())
        .insert_resource(InventoryItems::default())
        .insert_resource(GameMode::Playing)
        .insert_resource(CarriedItem(None))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenInfiniFactory Prototype".to_string(),
                resolution: (1280.0, 720.0).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(
            Startup,
            (setup_scene, load_world_on_startup, setup_ui).chain(),
        )
        .add_systems(
            Update,
            (
                camera_move,
                camera_look,
                inventory_input,
                placement_input,
                save_load_input,
                update_hover,
                inventory_slot_clicks,
                update_ui,
                sync_cursor_grab,
            ),
        )
        .run();
}

#[derive(Component)]
struct FlyCamera {
    yaw: f32,
    pitch: f32,
    velocity_y: f32,
    grounded: bool,
    sensitivity: f32,
}

#[derive(Component)]
struct BlockEntity;

#[derive(Component)]
struct HoverMarker;

#[derive(Component)]
struct HotbarText;

#[derive(Component)]
struct BackpackPanel;

#[derive(Component)]
struct PausePanel;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct SlotLabel;

#[derive(Component)]
struct CarriedLabel;

#[derive(Component, Clone, Copy)]
struct InventorySlot {
    area: SlotArea,
    index: usize,
}

#[derive(Resource, Default)]
struct WorldBlocks {
    blocks: HashMap<IVec3, BlockData>,
}

#[derive(Resource)]
struct PlacementState {
    selected: usize,
    facing: Facing,
    target: Option<TargetHit>,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected: 0,
            facing: Facing::North,
            target: None,
        }
    }
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
enum GameMode {
    Playing,
    Inventory,
    Paused,
}

#[derive(Resource)]
struct InventoryItems {
    hotbar: [Option<BlockKind>; HOTBAR_SLOTS],
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
struct CarriedItem(Option<BlockKind>);

#[derive(Clone, Copy, Eq, PartialEq)]
enum SlotArea {
    Hotbar,
    Backpack,
}

#[derive(Clone, Copy)]
struct TargetHit {
    pos: IVec3,
    normal: IVec3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct BlockData {
    kind: BlockKind,
    facing: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum BlockKind {
    Solid,
    Conveyor,
    Piston,
    Glass,
    Goal,
}

impl BlockKind {
    fn name(self) -> &'static str {
        match self {
            BlockKind::Solid => "Solid",
            BlockKind::Conveyor => "Conveyor",
            BlockKind::Piston => "Piston",
            BlockKind::Glass => "Glass",
            BlockKind::Goal => "Goal",
        }
    }

    fn material(self) -> Color {
        match self {
            BlockKind::Solid => Color::srgb(0.46, 0.48, 0.50),
            BlockKind::Conveyor => Color::srgb(0.10, 0.22, 0.28),
            BlockKind::Piston => Color::srgb(0.78, 0.55, 0.28),
            BlockKind::Glass => Color::srgba(0.55, 0.82, 0.95, 0.45),
            BlockKind::Goal => Color::srgb(0.35, 0.72, 0.42),
        }
    }

    fn is_directional(self) -> bool {
        matches!(self, BlockKind::Conveyor | BlockKind::Piston)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    fn rotate(self) -> Self {
        match self {
            Facing::North => Facing::East,
            Facing::East => Facing::South,
            Facing::South => Facing::West,
            Facing::West => Facing::North,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Facing::North => "North",
            Facing::East => "East",
            Facing::South => "South",
            Facing::West => "West",
        }
    }

    fn yaw(self) -> f32 {
        match self {
            Facing::North => 0.0,
            Facing::East => -std::f32::consts::FRAC_PI_2,
            Facing::South => std::f32::consts::PI,
            Facing::West => std::f32::consts::FRAC_PI_2,
        }
    }

    fn forward(self) -> Vec3 {
        match self {
            Facing::North => Vec3::new(0.0, 0.0, -1.0),
            Facing::East => Vec3::new(1.0, 0.0, 0.0),
            Facing::South => Vec3::new(0.0, 0.0, 1.0),
            Facing::West => Vec3::new(-1.0, 0.0, 0.0),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    blocks: Vec<SavedBlock>,
}

#[derive(Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.0, EYE_HEIGHT, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCamera {
            yaw: std::f32::consts::PI,
            pitch: -0.15,
            velocity_y: 0.0,
            grounded: false,
            sensitivity: 0.0025,
        },
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 4500.0,
            shadows_enabled: true,
            range: 40.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 9.0, 6.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.6, 0.0)),
        ..default()
    });

    let floor_mesh = meshes.add(Plane3d::default().mesh().size(32.0, 32.0));
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.24, 0.22),
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: floor_mesh,
        material: floor_material,
        ..default()
    });

    let marker_mesh = meshes.add(Cuboid::new(1.04, 1.04, 1.04));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.16),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: marker_mesh,
            material: marker_material,
            visibility: Visibility::Hidden,
            ..default()
        },
        HoverMarker,
    ));
}

fn load_world_on_startup(
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !load_world(&mut world) {
        seed_demo_world(&mut world);
    }
    rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
}

fn seed_demo_world(world: &mut WorldBlocks) {
    for x in -3..=3 {
        for z in -3..=3 {
            if (x + z) % 3 == 0 {
                world.blocks.insert(
                    IVec3::new(x, 0, z),
                    BlockData {
                        kind: BlockKind::Solid,
                        facing: Facing::North,
                    },
                );
            }
        }
    }
    world.blocks.insert(
        IVec3::new(0, 1, 0),
        BlockData {
            kind: BlockKind::Conveyor,
            facing: Facing::East,
        },
    );
    world.blocks.insert(
        IVec3::new(1, 1, 0),
        BlockData {
            kind: BlockKind::Piston,
            facing: Facing::South,
        },
    );
    world.blocks.insert(
        IVec3::new(2, 1, 0),
        BlockData {
            kind: BlockKind::Goal,
            facing: Facing::North,
        },
    );
}

fn setup_ui(mut commands: Commands) {
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
                panel.spawn(TextBundle {
                    text: Text::from_section(
                        "Inventory",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::srgb(0.94, 0.94, 0.92),
                            ..default()
                        },
                    ),
                    ..default()
                });

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

                panel.spawn(TextBundle {
                    text: Text::from_section(
                        "Click a slot to pick up or swap. Number keys select the hotbar.",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::srgb(0.78, 0.78, 0.76),
                            ..default()
                        },
                    ),
                    ..default()
                });
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
                panel.spawn(TextBundle {
                    text: Text::from_section(
                        "Paused",
                        TextStyle {
                            font_size: 30.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                });
                panel.spawn(TextBundle {
                    text: Text::from_section(
                        "Press ESC or left click to return to the game.",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(0.82, 0.84, 0.84),
                            ..default()
                        },
                    ),
                    ..default()
                });
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

fn camera_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<GameMode>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    if *mode != GameMode::Playing {
        return;
    }

    let Ok((mut camera, mut transform)) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    let yaw_rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;
    let right = yaw_rotation * Vec3::X;

    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }

    if direction.length_squared() > 0.0 {
        let horizontal = Vec3::new(direction.x, 0.0, direction.z).normalize();
        transform.translation += horizontal * PLAYER_SPEED * time.delta_seconds();
    }

    if keys.just_pressed(KeyCode::Space) && camera.grounded {
        camera.velocity_y = JUMP_SPEED;
        camera.grounded = false;
    }

    camera.velocity_y -= GRAVITY * time.delta_seconds();
    transform.translation.y += camera.velocity_y * time.delta_seconds();

    if transform.translation.y <= EYE_HEIGHT {
        transform.translation.y = EYE_HEIGHT;
        camera.velocity_y = 0.0;
        camera.grounded = true;
    }
}

fn camera_look(
    mode: Res<GameMode>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    let Ok((mut camera, mut transform)) = query.get_single_mut() else {
        return;
    };

    if *mode != GameMode::Playing {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    camera.yaw -= delta.x * camera.sensitivity;
    camera.pitch = (camera.pitch - delta.y * camera.sensitivity).clamp(-1.45, 1.45);
    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, camera.yaw) * Quat::from_axis_angle(Vec3::X, camera.pitch);
}

fn inventory_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mode: ResMut<GameMode>,
    mut placement: ResMut<PlacementState>,
) {
    if keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::KeyI) {
        *mode = if *mode == GameMode::Inventory {
            GameMode::Playing
        } else {
            GameMode::Inventory
        };
    }

    if keys.just_pressed(KeyCode::Escape) {
        *mode = if *mode == GameMode::Playing {
            GameMode::Paused
        } else {
            GameMode::Playing
        };
    }

    if *mode == GameMode::Paused && mouse_buttons.just_pressed(MouseButton::Left) {
        *mode = GameMode::Playing;
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ] {
        if keys.just_pressed(key) {
            placement.selected = index;
        }
    }

    if keys.just_pressed(KeyCode::KeyR) {
        placement.facing = placement.facing.rotate();
    }
}

fn placement_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    inventory: Res<InventoryItems>,
    mode: Res<GameMode>,
    placement: Res<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
) {
    if *mode != GameMode::Playing {
        return;
    }

    let Some(target) = placement.target else {
        return;
    };

    let mut changed = false;
    if mouse_buttons.just_pressed(MouseButton::Left) {
        world.blocks.remove(&target.pos);
        changed = true;
    }

    if mouse_buttons.just_pressed(MouseButton::Right) {
        let place_at = target.pos + target.normal;
        if place_at.y >= 0 && !world.blocks.contains_key(&place_at) {
            let Some(kind) = inventory.hotbar[placement.selected] else {
                return;
            };
            world.blocks.insert(
                place_at,
                BlockData {
                    kind,
                    facing: placement.facing,
                },
            );
            changed = true;
        }
    }

    if changed {
        for entity in &block_entities {
            commands.entity(entity).despawn_recursive();
        }
        rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
    }
}

fn save_load_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
) {
    if keys.just_pressed(KeyCode::F5) {
        save_world(&world);
    }

    if keys.just_pressed(KeyCode::F9) && load_world(&mut world) {
        for entity in &block_entities {
            commands.entity(entity).despawn_recursive();
        }
        rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
    }
}

fn inventory_slot_clicks(
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

fn update_hover(
    mut placement: ResMut<PlacementState>,
    mode: Res<GameMode>,
    camera: Query<&Transform, (With<FlyCamera>, Without<HoverMarker>)>,
    world: Res<WorldBlocks>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<HoverMarker>>,
) {
    if *mode != GameMode::Playing {
        placement.target = None;
        if let Ok((_, mut visibility)) = marker.get_single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Ok(camera_transform) = camera.get_single() else {
        return;
    };

    placement.target = raycast_blocks(
        camera_transform.translation,
        *camera_transform.forward(),
        &world,
    );

    let Ok((mut marker_transform, mut visibility)) = marker.get_single_mut() else {
        return;
    };

    if let Some(target) = placement.target {
        marker_transform.translation = grid_to_world(target.pos);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn update_ui(
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

fn sync_cursor_grab(mode: Res<GameMode>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if !mode.is_changed() {
        return;
    }

    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    if *mode == GameMode::Playing {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
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

fn rebuild_world(
    commands: &mut Commands,
    world: &WorldBlocks,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (pos, data) in &world.blocks {
        spawn_block(commands, meshes, materials, *pos, *data);
    }
}

fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: IVec3,
    data: BlockData,
) {
    let block_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
    let mut material = StandardMaterial {
        base_color: data.kind.material(),
        perceptual_roughness: 0.82,
        ..default()
    };
    if data.kind == BlockKind::Glass {
        material.alpha_mode = AlphaMode::Blend;
    }
    let block_material = materials.add(material);

    commands
        .spawn((
            PbrBundle {
                mesh: block_mesh,
                material: block_material,
                transform: Transform::from_translation(grid_to_world(pos)),
                ..default()
            },
            BlockEntity,
        ))
        .with_children(|parent| {
            if data.kind.is_directional() {
                let forward = data.facing.forward();
                let arrow_mesh = meshes.add(Cuboid::new(0.18, 0.08, 0.72));
                let arrow_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.95, 0.95, 0.38),
                    unlit: true,
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: arrow_mesh,
                    material: arrow_material,
                    transform: Transform {
                        translation: forward * 0.05 + Vec3::Y * 0.54,
                        rotation: Quat::from_rotation_y(data.facing.yaw()),
                        ..default()
                    },
                    ..default()
                });

                let nose_mesh = meshes.add(Cuboid::new(0.42, 0.10, 0.18));
                let nose_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.78, 0.25),
                    unlit: true,
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: nose_mesh,
                    material: nose_material,
                    transform: Transform {
                        translation: forward * 0.42 + Vec3::Y * 0.56,
                        rotation: Quat::from_rotation_y(data.facing.yaw()),
                        ..default()
                    },
                    ..default()
                });
            }

            if data.kind == BlockKind::Goal {
                let top_mesh = meshes.add(Cuboid::new(0.62, 0.08, 0.62));
                let top_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.75, 1.0, 0.55),
                    emissive: Color::srgb(0.12, 0.28, 0.08).into(),
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: top_mesh,
                    material: top_material,
                    transform: Transform::from_xyz(0.0, 0.55, 0.0),
                    ..default()
                });
            }
        });
}

fn grid_to_world(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.y as f32 + 0.5, pos.z as f32)
}

fn raycast_blocks(origin: Vec3, dir: Vec3, world: &WorldBlocks) -> Option<TargetHit> {
    let mut best: Option<(f32, TargetHit)> = None;

    for pos in world.blocks.keys() {
        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max) {
            if distance <= REACH && best.map_or(true, |(best_distance, _)| distance < best_distance)
            {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }

    best.map(|(_, hit)| hit)
}

fn ray_aabb(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<(f32, IVec3)> {
    let mut t_min = 0.0;
    let mut t_max = REACH;
    let mut normal = IVec3::ZERO;

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let dir_axis = dir[axis];
        let min_axis = min[axis];
        let max_axis = max[axis];

        if dir_axis.abs() < 0.0001 {
            if origin_axis < min_axis || origin_axis > max_axis {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / dir_axis;
        let mut t1 = (min_axis - origin_axis) * inv_dir;
        let mut t2 = (max_axis - origin_axis) * inv_dir;
        let mut axis_normal = IVec3::ZERO;

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            axis_normal[axis] = 1;
        } else {
            axis_normal[axis] = -1;
        }

        if t1 > t_min {
            t_min = t1;
            normal = axis_normal;
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    Some((t_min.max(0.0), normal))
}

fn save_world(world: &WorldBlocks) {
    let save = SaveFile {
        blocks: world
            .blocks
            .iter()
            .map(|(pos, data)| SavedBlock {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                data: *data,
            })
            .collect(),
    };

    if let Some(parent) = Path::new(SAVE_PATH).parent() {
        let _ = fs::create_dir_all(parent);
    }

    match ron::ser::to_string_pretty(&save, PrettyConfig::default()) {
        Ok(serialized) => {
            if let Err(error) = fs::write(SAVE_PATH, serialized) {
                warn!("Failed to save world: {error}");
            }
        }
        Err(error) => warn!("Failed to serialize world: {error}"),
    }
}

fn load_world(world: &mut WorldBlocks) -> bool {
    let Ok(contents) = fs::read_to_string(SAVE_PATH) else {
        return false;
    };
    let Ok(save) = ron::from_str::<SaveFile>(&contents) else {
        return false;
    };

    world.blocks.clear();
    for saved in save.blocks {
        world
            .blocks
            .insert(IVec3::new(saved.x, saved.y, saved.z), saved.data);
    }
    true
}
