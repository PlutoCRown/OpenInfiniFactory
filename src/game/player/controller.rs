use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::{Hdr, RenderTarget};
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::light::ShadowFilteringMethod;
use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::game::cameras::{
    GameplayCamera, GameplayViewImage, MENU_CLEAR, PlayingUiCamera, gameplay_view_size,
    new_gameplay_view_image,
};
use crate::game::scene_blocks::SceneBlockRegistry;
use crate::game::simulation::movement::PusherState;
use crate::game::state::{GameMode, GameSettings, PlayingUiState};
use crate::game::ui::UiRuntime;
use crate::game::world::grid::{WorldBlocks, grid_to_world};
use crate::game::world::rendering::{GameplayScene, environment_map_light};
use crate::shared::config::GameConfig;
use crate::shared::save::PuzzleLighting;

pub const EYE_HEIGHT: f32 = 1.7;
pub const PLAYER_RADIUS: f32 = 0.28;
pub const PLAYER_HEIGHT: f32 = EYE_HEIGHT;
const FLOOR_TOP_Y: f32 = 1.0;
const SPAWN_EYE_Y: f32 = FLOOR_TOP_Y + EYE_HEIGHT + 0.08;
const PLAYER_SPEED: f32 = 5.5;
const FLY_SPEED: f32 = 7.0;
const GRAVITY: f32 = 18.0;
/// Vertical travel (eye/camera position) needed to step onto the next 1-block tier.
const ONE_BLOCK_JUMP_HEIGHT: f32 = 1.5;
const DOUBLE_TAP_WINDOW: f32 = 0.28;
const AABB_EPSILON: f32 = 0.001;
/// 水平被挡时允许的最大抬升（斜坡/半砖）；整格高墙仍需跳
const STEP_HEIGHT: f32 = 0.55;
/// 鼠标视角基础灵敏度，X/Y 轴倍率由设置中的 mouse_sensitivity_x/y 控制。
const BASE_MOUSE_SENSITIVITY: f32 = 0.0025;

#[derive(Component)]
pub struct FlyCamera {
    yaw: f32,
    pitch: f32,
    velocity_y: f32,
    grounded: bool,
    flying: bool,
    last_space_press: f32,
}

pub fn spawn_player(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    window: Query<&Window, With<PrimaryWindow>>,
    config: Res<GameConfig>,
    lighting: Res<PuzzleLighting>,
) {
    let (width, height) = window
        .single()
        .map(gameplay_view_size)
        .unwrap_or((1280, 720));
    let image_handle = images.add(new_gameplay_view_image(width, height));
    commands.insert_resource(GameplayViewImage(image_handle.clone()));

    let clear_color = if config.skybox_enabled {
        ClearColorConfig::Custom(Color::BLACK)
    } else {
        ClearColorConfig::Custom(MENU_CLEAR)
    };

    commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: 0,
                clear_color,
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            Transform::from_xyz(0.5, SPAWN_EYE_Y + 1.2, 10.5)
                .looking_at(Vec3::new(0.5, 0.8, 0.5), Vec3::Y),
            FlyCamera {
                yaw: std::f32::consts::PI,
                pitch: -0.15,
                velocity_y: 0.0,
                grounded: false,
                flying: false,
                last_space_press: -10.0,
            },
            GameplayCamera,
            GameplayScene,
            environment_map_light(&mut images, &lighting),
        ))
        .insert((
            Hdr,
            Msaa::Off,
            Tonemapping::TonyMcMapface,
            DebandDither::Enabled,
            // 高阈值：只让电线充能条 / 焊点（自发光 ≫ 环境光）泛光
            Bloom {
                intensity: 0.7,
                low_frequency_boost: 0.85,
                low_frequency_boost_curvature: 0.85,
                high_pass_frequency: 0.85,
                prefilter: BloomPrefilter {
                    threshold: 10.0,
                    threshold_softness: 0.5,
                },
                composite_mode: BloomCompositeMode::Additive,
                ..Bloom::NATURAL
            },
            ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                ..default()
            },
            TemporalAntiAliasing::default(),
            TemporalJitter::default(),
            DepthPrepass,
            NormalPrepass,
            MotionVectorPrepass,
            ShadowFilteringMethod::Temporal,
        ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Msaa::Off,
        IsDefaultUiCamera,
        PlayingUiCamera,
        GameplayScene,
    ));
}

use crate::shared::save::PlayerSave;

pub fn capture_player_save(camera: &FlyCamera, transform: &Transform) -> PlayerSave {
    PlayerSave {
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        yaw: camera.yaw,
        pitch: camera.pitch,
        flying: camera.flying,
    }
}

pub fn apply_player_save(camera: &mut FlyCamera, transform: &mut Transform, save: &PlayerSave) {
    transform.translation = Vec3::new(save.x, save.y, save.z);
    camera.yaw = save.yaw;
    camera.pitch = save.pitch;
    camera.flying = save.flying;
    camera.velocity_y = 0.0;
    camera.grounded = false;
    camera.last_space_press = -10.0;
    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, camera.yaw) * Quat::from_axis_angle(Vec3::X, camera.pitch);
}

pub fn apply_pending_player_spawn(
    mut pending: ResMut<crate::game::state::PendingPlayerSpawn>,
    mut player: Query<(&mut FlyCamera, &mut Transform)>,
) {
    let Some(save) = pending.0.take() else {
        return;
    };
    let Ok((mut camera, mut transform)) = player.single_mut() else {
        return;
    };
    apply_player_save(&mut camera, &mut transform, &save);
}

pub fn camera_move(
    time: Res<Time>,
    input: Res<crate::game::input::GameplayInputState>,
    settings: Res<GameSettings>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    pusher_state: Res<PusherState>,
    scene_registry: Res<SceneBlockRegistry>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    if *mode.get() != GameMode::Playing || !playing_ui.active_play() || ui_runtime.blocks_gameplay()
    {
        return;
    }

    let Ok((mut camera, mut transform)) = query.single_mut() else {
        return;
    };

    let now = time.elapsed_secs();

    if input.jump.just_pressed {
        if now - camera.last_space_press <= DOUBLE_TAP_WINDOW {
            camera.flying = !camera.flying;
            camera.velocity_y = 0.0;
            camera.grounded = false;
        } else if camera.grounded {
            camera.velocity_y = jump_speed(settings.gravity_scale);
            camera.grounded = false;
        }
        camera.last_space_press = now;
    }

    let mut direction = Vec3::ZERO;
    let yaw_rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;
    let right = yaw_rotation * Vec3::X;

    direction += forward * input.move_axis.y;
    direction += right * input.move_axis.x;

    if direction.length_squared() > 0.0 {
        let horizontal = Vec3::new(direction.x, 0.0, direction.z).normalize();
        let speed = if camera.flying {
            FLY_SPEED
        } else {
            PLAYER_SPEED
        };
        let delta = horizontal * speed * time.delta_secs();
        move_with_collision(
            &mut transform.translation,
            delta,
            &world,
            &pusher_state,
            &scene_registry,
            true,
            camera.grounded && !camera.flying,
        );
    }

    if camera.flying {
        let mut vertical = 0.0;
        if input.fly_up {
            vertical += 1.0;
        }
        if input.fly_down {
            vertical -= 1.0;
        }
        if vertical != 0.0 {
            let before_y = transform.translation.y;
            move_with_collision(
                &mut transform.translation,
                Vec3::Y * vertical * FLY_SPEED * time.delta_secs(),
                &world,
                &pusher_state,
                &scene_registry,
                false,
                false,
            );
            if vertical < 0.0
                && (transform.translation.y == before_y
                    || is_supported(
                        transform.translation,
                        &world,
                        &pusher_state,
                        &scene_registry,
                    ))
            {
                camera.flying = false;
                camera.grounded = true;
                camera.velocity_y = 0.0;
            }
        }
    } else {
        camera.velocity_y -= GRAVITY * settings.gravity_scale * time.delta_secs();
        let vertical_delta = Vec3::Y * camera.velocity_y * time.delta_secs();
        let before = transform.translation;
        move_with_collision(
            &mut transform.translation,
            vertical_delta,
            &world,
            &pusher_state,
            &scene_registry,
            false,
            false,
        );

        if transform.translation.y != before.y && camera.velocity_y > 0.0 {
            camera.grounded = false;
        } else if transform.translation.y == before.y && camera.velocity_y <= 0.0 {
            camera.velocity_y = 0.0;
            camera.grounded = is_supported(
                transform.translation,
                &world,
                &pusher_state,
                &scene_registry,
            );
        } else {
            camera.grounded = is_supported(
                transform.translation,
                &world,
                &pusher_state,
                &scene_registry,
            );
        }
    }

    if transform.translation.y < SPAWN_EYE_Y {
        transform.translation.y = SPAWN_EYE_Y;
        camera.velocity_y = 0.0;
        camera.grounded = true;
    }
}

pub fn camera_look(
    keys: Res<ButtonInput<KeyCode>>,
    input: Res<crate::game::input::GameplayInputState>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    let Ok((mut camera, mut transform)) = query.single_mut() else {
        return;
    };

    if *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || ui_runtime.blocks_gameplay()
        || alt_pressed(&keys)
    {
        return;
    }

    let delta = input.look_delta;
    if delta == Vec2::ZERO {
        return;
    }

    camera.yaw -= delta.x * BASE_MOUSE_SENSITIVITY * settings.mouse_sensitivity_x;
    camera.pitch = (camera.pitch - delta.y * BASE_MOUSE_SENSITIVITY * settings.mouse_sensitivity_y)
        .clamp(-1.45, 1.45);
    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, camera.yaw) * Quat::from_axis_angle(Vec3::X, camera.pitch);
}

pub fn sync_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    touch: Res<crate::shared::touch_profile::TouchProfile>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    mut windows: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let Ok((mut window, mut cursor)) = windows.single_mut() else {
        return;
    };

    if touch.enabled {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
        return;
    }

    let want_lock = *mode.get() == GameMode::Playing
        && playing_ui.active_play()
        && !ui_runtime.blocks_gameplay()
        && !alt_pressed(&keys);
    if want_lock {
        let just_locked = cursor.grab_mode != CursorGrabMode::Locked;
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
        // 锁回准心时把系统光标移到窗口中心，避免解锁后再锁时光标还在边上
        if just_locked {
            let center = window.size() * 0.5;
            window.set_cursor_position(Some(center));
        }
    } else {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn alt_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)
}

/// Initial upward speed so apex height stays at one block for any gravity scale.
/// Continuous physics: h = v² / (2g)  =>  v = sqrt(2 * g * h).
fn jump_speed(gravity_scale: f32) -> f32 {
    (2.0 * GRAVITY * gravity_scale * ONE_BLOCK_JUMP_HEIGHT).sqrt()
}

pub fn teleport_player_preserve_offset(
    source: IVec3,
    partner: IVec3,
    transform: &mut Transform,
    camera: &mut FlyCamera,
) {
    let offset = transform.translation - grid_to_world(source);
    transform.translation = grid_to_world(partner) + offset;
    camera.velocity_y = 0.0;
}

pub fn player_intersects_block(position: Vec3, block: IVec3) -> bool {
    let (player_min, player_max) = player_aabb(position);
    let block_min = block.as_vec3();
    let block_max = block_min + Vec3::ONE;
    aabb_intersects(player_min, player_max, block_min, block_max)
}

pub fn player_collision_box(position: Vec3) -> (Vec3, Vec3) {
    player_aabb(position)
}

fn move_with_collision(
    position: &mut Vec3,
    delta: Vec3,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
    allow_step_up: bool,
    ground_snap: bool,
) {
    let overlap0 = collision_overlap_score(*position, world, pusher_state, scene_registry);

    // X：可走则走；否则在 STEP_HEIGHT 内二分最小抬升（贴斜面，而非整级蹦）
    let mut next = *position;
    next.x += delta.x;
    if can_move_to(next, overlap0, world, pusher_state, scene_registry) {
        position.x = next.x;
    } else if allow_step_up && delta.x.abs() > AABB_EPSILON {
        let base_y = position.y;
        let mut probe = *position;
        probe.x += delta.x;
        probe.y = base_y + STEP_HEIGHT;
        if can_move_to(probe, overlap0, world, pusher_state, scene_registry) {
            let mut lo = 0.0;
            let mut hi = STEP_HEIGHT;
            for _ in 0..12 {
                let mid = (lo + hi) * 0.5;
                probe.y = base_y + mid;
                if can_move_to(probe, overlap0, world, pusher_state, scene_registry) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            position.x += delta.x;
            position.y = base_y + hi;
        }
    }

    let overlap1 = collision_overlap_score(*position, world, pusher_state, scene_registry);

    // Z
    next = *position;
    next.z += delta.z;
    if can_move_to(next, overlap1, world, pusher_state, scene_registry) {
        position.z = next.z;
    } else if allow_step_up && delta.z.abs() > AABB_EPSILON {
        let base_y = position.y;
        let mut probe = *position;
        probe.z += delta.z;
        probe.y = base_y + STEP_HEIGHT;
        if can_move_to(probe, overlap1, world, pusher_state, scene_registry) {
            let mut lo = 0.0;
            let mut hi = STEP_HEIGHT;
            for _ in 0..12 {
                let mid = (lo + hi) * 0.5;
                probe.y = base_y + mid;
                if can_move_to(probe, overlap1, world, pusher_state, scene_registry) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            position.z += delta.z;
            position.y = base_y + hi;
        }
    }

    // 贴地：步行着地时把多余抬升收回斜面/地面上，避免一格一格的落差感
    if ground_snap {
        let start_y = position.y;
        let mut probe = *position;
        probe.y = start_y - STEP_HEIGHT;
        if collides(*position, world, pusher_state, scene_registry) {
            // 已陷入则交给 overlap 脱困，不硬拽
        } else if collides(probe, world, pusher_state, scene_registry)
            || is_supported(
                Vec3::new(position.x, start_y - STEP_HEIGHT + 0.02, position.z),
                world,
                pusher_state,
                scene_registry,
            )
        {
            let mut lo = 0.0;
            let mut hi = STEP_HEIGHT;
            for _ in 0..12 {
                let mid = (lo + hi) * 0.5;
                probe.y = start_y - mid;
                if collides(probe, world, pusher_state, scene_registry) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            // lo ≈ 最大不碰撞下沉量；再微抬避免贴面抖动
            position.y = start_y - lo + AABB_EPSILON * 2.0;
        }
    }

    let overlap2 = collision_overlap_score(*position, world, pusher_state, scene_registry);

    // Y（重力 / 飞行竖移）
    next = *position;
    next.y += delta.y;
    if can_move_to(next, overlap2, world, pusher_state, scene_registry) {
        position.y = next.y;
    }
}

fn player_hits_block(
    player_min: Vec3,
    player_max: Vec3,
    block_pos: IVec3,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
) -> bool {
    if pusher_state
        .extended_head_positions(world)
        .contains(&block_pos)
    {
        let block_min = block_pos.as_vec3();
        return aabb_intersects(player_min, player_max, block_min, block_min + Vec3::ONE);
    }
    let Some(block) = world.blocks.get(&block_pos) else {
        return false;
    };
    if !block.kind.has_collision() {
        return false;
    }
    if let Some(tris) = scene_registry.collision_tris(block.kind) {
        return aabb_hits_collision_mesh(player_min, player_max, block_pos, block.facing, tris);
    }
    let block_min = block_pos.as_vec3();
    aabb_intersects(player_min, player_max, block_min, block_min + Vec3::ONE)
}

fn collides(
    position: Vec3,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
) -> bool {
    let (min, max) = player_aabb(position);

    let min_block = min.floor().as_ivec3();
    let max_block = (max - Vec3::splat(AABB_EPSILON)).floor().as_ivec3();

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                if player_hits_block(
                    min,
                    max,
                    IVec3::new(x, y, z),
                    world,
                    pusher_state,
                    scene_registry,
                ) {
                    return true;
                }
            }
        }
    }

    false
}

fn can_move_to(
    next: Vec3,
    current_overlap: f32,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
) -> bool {
    if !collides(next, world, pusher_state, scene_registry) {
        return true;
    }

    current_overlap > 0.0
        && collision_overlap_score(next, world, pusher_state, scene_registry) < current_overlap
}

fn collision_overlap_score(
    position: Vec3,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
) -> f32 {
    let (min, max) = player_aabb(position);
    let min_block = min.floor().as_ivec3();
    let max_block = (max - Vec3::splat(AABB_EPSILON)).floor().as_ivec3();
    let mut score = 0.0;

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                let block_pos = IVec3::new(x, y, z);
                if !player_hits_block(min, max, block_pos, world, pusher_state, scene_registry) {
                    continue;
                }

                let (block_min, block_max) =
                    block_collision_aabb(block_pos, world, pusher_state, scene_registry);
                let overlap = (max.min(block_max) - min.max(block_min)).max(Vec3::ZERO);
                score += overlap.x * overlap.y * overlap.z;
            }
        }
    }

    score
}

fn is_supported(
    position: Vec3,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
) -> bool {
    let (min, max) = player_aabb(position);
    let probe_min = Vec3::new(min.x, min.y - 0.04, min.z);
    let probe_max = Vec3::new(max.x, min.y, max.z);
    let min_x = probe_min.x.floor() as i32;
    let max_x = (probe_max.x - AABB_EPSILON).floor() as i32;
    let min_z = probe_min.z.floor() as i32;
    let max_z = (probe_max.z - AABB_EPSILON).floor() as i32;
    let min_y = probe_min.y.floor() as i32;
    let max_y = (probe_max.y - AABB_EPSILON).floor() as i32;

    if max_y < 0 {
        return false;
    }

    for x in min_x..=max_x {
        for y in min_y.max(0)..=max_y {
            for z in min_z..=max_z {
                if player_hits_block(
                    probe_min,
                    probe_max,
                    IVec3::new(x, y, z),
                    world,
                    pusher_state,
                    scene_registry,
                ) {
                    return true;
                }
            }
        }
    }

    false
}

/// 碰撞体世界 AABB（网格用顶点外包；否则整格）
fn block_collision_aabb(
    block_pos: IVec3,
    world: &WorldBlocks,
    pusher_state: &PusherState,
    scene_registry: &SceneBlockRegistry,
) -> (Vec3, Vec3) {
    let full_min = block_pos.as_vec3();
    let full_max = full_min + Vec3::ONE;
    if pusher_state
        .extended_head_positions(world)
        .contains(&block_pos)
    {
        return (full_min, full_max);
    }
    let Some(block) = world.blocks.get(&block_pos) else {
        return (full_min, full_max);
    };
    let Some(tris) = scene_registry.collision_tris(block.kind) else {
        return (full_min, full_max);
    };
    let center = grid_to_world(block_pos);
    let rot = Quat::from_rotation_y(block.facing.yaw());
    let mut bmin = Vec3::splat(f32::INFINITY);
    let mut bmax = Vec3::splat(f32::NEG_INFINITY);
    for tri in tris {
        for v in tri {
            let w = center + rot * *v;
            bmin = bmin.min(w);
            bmax = bmax.max(w);
        }
    }
    (bmin, bmax)
}

/// 玩家 AABB 是否碰到局部 collision 网格（含朝向旋转）
fn aabb_hits_collision_mesh(
    player_min: Vec3,
    player_max: Vec3,
    block_pos: IVec3,
    facing: crate::game::world::direction::Facing,
    tris: &[[Vec3; 3]],
) -> bool {
    let center = grid_to_world(block_pos);
    let rot = Quat::from_rotation_y(facing.yaw());
    for tri in tris {
        let a = center + rot * tri[0];
        let b = center + rot * tri[1];
        let c = center + rot * tri[2];
        if aabb_intersects_triangle(player_min, player_max, a, b, c) {
            return true;
        }
    }
    false
}

/// AABB–三角形相交（分离轴）
fn aabb_intersects_triangle(amin: Vec3, amax: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> bool {
    let tmin = v0.min(v1).min(v2);
    let tmax = v0.max(v1).max(v2);
    if !aabb_intersects(amin, amax, tmin, tmax) {
        return false;
    }

    let center = (amin + amax) * 0.5;
    let extents = (amax - amin) * 0.5;
    let v0 = v0 - center;
    let v1 = v1 - center;
    let v2 = v2 - center;
    let e0 = v1 - v0;
    let e1 = v2 - v1;
    let e2 = v0 - v2;

    let axis_test = |axis: Vec3| -> bool {
        if axis.length_squared() < 1e-12 {
            return true;
        }
        let p0 = v0.dot(axis);
        let p1 = v1.dot(axis);
        let p2 = v2.dot(axis);
        let r = extents.x * axis.x.abs() + extents.y * axis.y.abs() + extents.z * axis.z.abs();
        let min_p = p0.min(p1).min(p2);
        let max_p = p0.max(p1).max(p2);
        max_p >= -r && min_p <= r
    };

    // 9 个边叉乘轴
    for edge in [e0, e1, e2] {
        if !axis_test(Vec3::new(0.0, -edge.z, edge.y)) {
            return false;
        }
        if !axis_test(Vec3::new(edge.z, 0.0, -edge.x)) {
            return false;
        }
        if !axis_test(Vec3::new(-edge.y, edge.x, 0.0)) {
            return false;
        }
    }
    // AABB 三轴
    if !axis_test(Vec3::X) || !axis_test(Vec3::Y) || !axis_test(Vec3::Z) {
        return false;
    }
    // 三角形法线
    let normal = e0.cross(e1);
    axis_test(normal)
}

fn player_aabb(position: Vec3) -> (Vec3, Vec3) {
    let min = Vec3::new(
        position.x - PLAYER_RADIUS,
        position.y - PLAYER_HEIGHT,
        position.z - PLAYER_RADIUS,
    );
    let max = Vec3::new(
        position.x + PLAYER_RADIUS,
        position.y - 0.05,
        position.z + PLAYER_RADIUS,
    );
    (min, max)
}

fn aabb_intersects(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
    a_min.x < b_max.x
        && a_max.x > b_min.x
        && a_min.y < b_max.y
        && a_max.y > b_min.y
        && a_min.z < b_max.z
        && a_max.z > b_min.z
}
