use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SAVE_PATH: &str = "saves/world.ron";
const REACH: f32 = 8.0;
const BLOCK_SIZE: f32 = 1.0;
const HOTBAR: [BlockKind; 5] = [
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
        .insert_resource(InventoryOpen(false))
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
                update_ui,
            ),
        )
        .run();
}

#[derive(Component)]
struct FlyCamera {
    yaw: f32,
    pitch: f32,
    speed: f32,
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

#[derive(Resource)]
struct InventoryOpen(bool);

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
            transform: Transform::from_xyz(5.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCamera {
            yaw: -0.58,
            pitch: -0.45,
            speed: 7.0,
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
            root.spawn(TextBundle {
                text: Text::from_section(
                    "+",
                    TextStyle {
                        font_size: 28.0,
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
            });

            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(18.0),
                        bottom: Val::Px(18.0),
                        ..default()
                    },
                    ..default()
                },
                HotbarText,
            ));

            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(480.0),
                        height: Val::Px(260.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-240.0),
                            top: Val::Px(-130.0),
                            ..default()
                        },
                        padding: UiRect::all(Val::Px(20.0)),
                        display: Display::None,
                        ..default()
                    },
                    background_color: Color::srgba(0.05, 0.06, 0.07, 0.88).into(),
                    ..default()
                },
                BackpackPanel,
            ))
            .with_children(|panel| {
                panel.spawn(TextBundle {
                    text: Text::from_section(
                        "Backpack\n\n1 Solid\n2 Conveyor\n3 Piston\n4 Glass\n5 Goal\n\nR rotates directional blocks. F5 saves, F9 loads.",
                        TextStyle {
                            font_size: 22.0,
                            color: Color::srgb(0.92, 0.94, 0.94),
                            ..default()
                        },
                    ),
                    ..default()
                });
            });
        });
}

fn camera_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&FlyCamera, &mut Transform)>,
) {
    let Ok((camera, mut transform)) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    let forward = transform.forward();
    let right = transform.right();

    if keys.pressed(KeyCode::KeyW) {
        direction += *forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= *forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += *right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= *right;
    }
    if keys.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }

    if direction.length_squared() > 0.0 {
        transform.translation += direction.normalize() * camera.speed * time.delta_seconds();
    }
}

fn camera_look(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    let Ok((mut camera, mut transform)) = query.get_single_mut() else {
        return;
    };

    if !mouse_buttons.pressed(MouseButton::Right) {
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
    mut inventory_open: ResMut<InventoryOpen>,
    mut placement: ResMut<PlacementState>,
) {
    if keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::KeyI) {
        inventory_open.0 = !inventory_open.0;
    }
    if keys.just_pressed(KeyCode::Escape) {
        inventory_open.0 = false;
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
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
    placement: Res<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_entities: Query<Entity, With<BlockEntity>>,
) {
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
            world.blocks.insert(
                place_at,
                BlockData {
                    kind: HOTBAR[placement.selected],
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

fn update_hover(
    mut placement: ResMut<PlacementState>,
    camera: Query<&Transform, (With<FlyCamera>, Without<HoverMarker>)>,
    world: Res<WorldBlocks>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<HoverMarker>>,
) {
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
    inventory_open: Res<InventoryOpen>,
    mut hotbar: Query<&mut Text, With<HotbarText>>,
    mut panels: Query<&mut Style, With<BackpackPanel>>,
) {
    if placement.is_changed() {
        if let Ok(mut text) = hotbar.get_single_mut() {
            let mut line = String::from("Hotbar: ");
            for (index, kind) in HOTBAR.iter().enumerate() {
                if index == placement.selected {
                    line.push_str(&format!("[{}:{}] ", index + 1, kind.name()));
                } else {
                    line.push_str(&format!("{}:{} ", index + 1, kind.name()));
                }
            }
            line.push_str(&format!("  Facing: {}", placement.facing.name()));
            text.sections[0].value = line;
        }
    }

    if inventory_open.is_changed() {
        for mut style in &mut panels {
            style.display = if inventory_open.0 {
                Display::Flex
            } else {
                Display::None
            };
        }
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
