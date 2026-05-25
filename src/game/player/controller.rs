use bevy::core_pipeline::experimental::taa::TemporalAntiAliasSettings;
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{
    ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    ShadowFilteringMethod,
};
use bevy::prelude::*;
use bevy::render::camera::TemporalJitter;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::game::state::GameMode;
use crate::game::world::grid::WorldBlocks;
use crate::shared::config::{ConfigAction, GameConfig};

pub const EYE_HEIGHT: f32 = 1.7;
pub const PLAYER_RADIUS: f32 = 0.28;
pub const PLAYER_HEIGHT: f32 = EYE_HEIGHT;
const PLAYER_SPEED: f32 = 5.5;
const FLY_SPEED: f32 = 7.0;
const JUMP_SPEED: f32 = 6.5;
const GRAVITY: f32 = 18.0;
const DOUBLE_TAP_WINDOW: f32 = 0.28;
const AABB_EPSILON: f32 = 0.001;

#[derive(Component)]
pub struct FlyCamera {
    yaw: f32,
    pitch: f32,
    velocity_y: f32,
    grounded: bool,
    flying: bool,
    last_space_press: f32,
    sensitivity: f32,
}

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::SomewhatBoringDisplayTransform,
            deband_dither: DebandDither::Enabled,
            transform: Transform::from_xyz(3.0, EYE_HEIGHT, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ScreenSpaceAmbientOcclusionSettings {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
        },
        TemporalAntiAliasSettings::default(),
        TemporalJitter::default(),
        DepthPrepass,
        NormalPrepass,
        MotionVectorPrepass,
        ShadowFilteringMethod::Temporal,
        FlyCamera {
            yaw: std::f32::consts::PI,
            pitch: -0.15,
            velocity_y: 0.0,
            grounded: false,
            flying: false,
            last_space_press: -10.0,
            sensitivity: 0.0025,
        },
    ));
}

pub fn camera_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mode: Res<GameMode>,
    world: Res<WorldBlocks>,
    mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
    if *mode != GameMode::Playing {
        return;
    }

    let Ok((mut camera, mut transform)) = query.get_single_mut() else {
        return;
    };

    let now = time.elapsed_seconds();
    let bindings = &config.key_bindings;
    let jump_key = bindings.jump_or_fly_up.key_code();
    let fly_down_key = bindings.fly_down.key_code();

    if keys.just_pressed(jump_key) {
        if now - camera.last_space_press <= DOUBLE_TAP_WINDOW {
            camera.flying = !camera.flying;
            camera.velocity_y = 0.0;
            camera.grounded = false;
        } else if camera.grounded {
            camera.velocity_y = JUMP_SPEED;
            camera.grounded = false;
        }
        camera.last_space_press = now;
    }

    let mut direction = Vec3::ZERO;
    let yaw_rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;
    let right = yaw_rotation * Vec3::X;

    if keys.pressed(config.key(ConfigAction::Forward).key_code()) {
        direction += forward;
    }
    if keys.pressed(config.key(ConfigAction::Backward).key_code()) {
        direction -= forward;
    }
    if keys.pressed(config.key(ConfigAction::Right).key_code()) {
        direction += right;
    }
    if keys.pressed(config.key(ConfigAction::Left).key_code()) {
        direction -= right;
    }

    if direction.length_squared() > 0.0 {
        let horizontal = Vec3::new(direction.x, 0.0, direction.z).normalize();
        let speed = if camera.flying {
            FLY_SPEED
        } else {
            PLAYER_SPEED
        };
        let delta = horizontal * speed * time.delta_seconds();
        move_with_collision(&mut transform.translation, delta, &world);
    }

    if camera.flying {
        let mut vertical = 0.0;
        if keys.pressed(jump_key) {
            vertical += 1.0;
        }
        if keys.pressed(fly_down_key) {
            vertical -= 1.0;
        }
        if vertical != 0.0 {
            move_with_collision(
                &mut transform.translation,
                Vec3::Y * vertical * FLY_SPEED * time.delta_seconds(),
                &world,
            );
        }
    } else {
        camera.velocity_y -= GRAVITY * time.delta_seconds();
        let vertical_delta = Vec3::Y * camera.velocity_y * time.delta_seconds();
        let before = transform.translation;
        move_with_collision(&mut transform.translation, vertical_delta, &world);

        if transform.translation.y != before.y && camera.velocity_y > 0.0 {
            camera.grounded = false;
        } else if transform.translation.y == before.y && camera.velocity_y <= 0.0 {
            camera.velocity_y = 0.0;
            camera.grounded = is_supported(transform.translation, &world);
        } else {
            camera.grounded = is_supported(transform.translation, &world);
        }
    }

    if transform.translation.y < EYE_HEIGHT {
        transform.translation.y = EYE_HEIGHT;
        camera.velocity_y = 0.0;
        camera.grounded = true;
    }
}

pub fn camera_look(
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

pub fn sync_cursor_grab(mode: Res<GameMode>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
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

pub fn player_intersects_block(position: Vec3, block: IVec3) -> bool {
    let (player_min, player_max) = player_aabb(position);
    let block_min = block.as_vec3();
    let block_max = block_min + Vec3::ONE;
    aabb_intersects(player_min, player_max, block_min, block_max)
}

pub fn player_collision_box(position: Vec3) -> (Vec3, Vec3) {
    player_aabb(position)
}

fn move_with_collision(position: &mut Vec3, delta: Vec3, world: &WorldBlocks) {
    let mut next = *position;
    next.x += delta.x;
    if !collides(next, world) {
        position.x = next.x;
    }

    next = *position;
    next.z += delta.z;
    if !collides(next, world) {
        position.z = next.z;
    }

    next = *position;
    next.y += delta.y;
    if !collides(next, world) {
        position.y = next.y;
    }
}

fn collides(position: Vec3, world: &WorldBlocks) -> bool {
    let (min, max) = player_aabb(position);

    let min_block = min.floor().as_ivec3();
    let max_block = (max - Vec3::splat(AABB_EPSILON)).floor().as_ivec3();

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                if y >= 0 && world.blocks.contains_key(&IVec3::new(x, y, z)) {
                    return true;
                }
            }
        }
    }

    false
}

fn is_supported(position: Vec3, world: &WorldBlocks) -> bool {
    let (min, max) = player_aabb(position);
    let probe_y = min.y - 0.04;
    let min_x = min.x.floor() as i32;
    let max_x = (max.x - AABB_EPSILON).floor() as i32;
    let min_z = min.z.floor() as i32;
    let max_z = (max.z - AABB_EPSILON).floor() as i32;
    let y = probe_y.floor() as i32;

    if y < 0 {
        return false;
    }

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            if world.blocks.contains_key(&IVec3::new(x, y, z)) {
                return true;
            }
        }
    }

    false
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
