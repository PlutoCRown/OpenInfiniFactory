use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::world::{player_feet_block, WorldBlocks};
use crate::GameMode;

pub const EYE_HEIGHT: f32 = 1.7;
const PLAYER_SPEED: f32 = 5.5;
const JUMP_SPEED: f32 = 6.5;
const GRAVITY: f32 = 18.0;
const PLAYER_RADIUS: f32 = 0.28;

#[derive(Component)]
pub struct FlyCamera {
    yaw: f32,
    pitch: f32,
    velocity_y: f32,
    grounded: bool,
    sensitivity: f32,
}

pub fn spawn_player(mut commands: Commands) {
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
}

pub fn camera_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
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
        let delta = horizontal * PLAYER_SPEED * time.delta_seconds();
        try_move_horizontally(&mut transform.translation, delta, &world);
    }

    if keys.just_pressed(KeyCode::Space) && camera.grounded {
        camera.velocity_y = JUMP_SPEED;
        camera.grounded = false;
    }

    camera.velocity_y -= GRAVITY * time.delta_seconds();
    transform.translation.y += camera.velocity_y * time.delta_seconds();

    let feet = player_feet_block(transform.translation);
    let support_y = highest_support_y(transform.translation, &world).unwrap_or(0.0);
    let minimum_eye_y = support_y + EYE_HEIGHT;
    if transform.translation.y <= minimum_eye_y && world.blocks.contains_key(&feet) {
        transform.translation.y = minimum_eye_y;
        camera.velocity_y = 0.0;
        camera.grounded = true;
    } else if transform.translation.y <= EYE_HEIGHT {
        transform.translation.y = EYE_HEIGHT;
        camera.velocity_y = 0.0;
        camera.grounded = true;
    } else {
        camera.grounded = false;
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

fn try_move_horizontally(position: &mut Vec3, delta: Vec3, world: &WorldBlocks) {
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
}

fn collides(position: Vec3, world: &WorldBlocks) -> bool {
    let min = Vec3::new(
        position.x - PLAYER_RADIUS,
        position.y - EYE_HEIGHT,
        position.z - PLAYER_RADIUS,
    );
    let max = Vec3::new(
        position.x + PLAYER_RADIUS,
        position.y,
        position.z + PLAYER_RADIUS,
    );

    let min_block = IVec3::new(
        min.x.floor() as i32,
        min.y.floor() as i32,
        min.z.floor() as i32,
    );
    let max_block = IVec3::new(
        max.x.floor() as i32,
        max.y.floor() as i32,
        max.z.floor() as i32,
    );

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                if y >= 1 && world.blocks.contains_key(&IVec3::new(x, y, z)) {
                    return true;
                }
            }
        }
    }

    false
}

fn highest_support_y(position: Vec3, world: &WorldBlocks) -> Option<f32> {
    let x = position.x.floor() as i32;
    let z = position.z.floor() as i32;
    let feet_y = (position.y - EYE_HEIGHT + 0.08).floor() as i32;
    let mut highest = None;

    for y in 0..=feet_y {
        if world.blocks.contains_key(&IVec3::new(x, y, z)) {
            highest = Some(y as f32 + 1.0);
        }
    }

    highest
}
