use crate::game::world::direction::Facing;
use crate::game::world::grid::grid_to_world;
use bevy::prelude::*;

pub const EDIT_ANIMATION_SECONDS: f32 = 0.3;
pub const SIMULATION_TURN_SECONDS: f32 = 0.5;

#[derive(Clone, Copy)]
pub enum AnimationEasing {
    Linear,
    SmoothStep,
}

#[derive(Clone, Copy)]
pub struct AnimationTiming {
    pub duration: f32,
    pub easing: AnimationEasing,
}

impl AnimationTiming {
    pub const fn edit() -> Self {
        Self {
            duration: EDIT_ANIMATION_SECONDS,
            easing: AnimationEasing::SmoothStep,
        }
    }

    pub const fn simulation(duration: f32) -> Self {
        Self {
            duration,
            easing: AnimationEasing::Linear,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BlockAnimation {
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub from_facing: Facing,
    pub to_facing: Facing,
    pub kind: BlockAnimationKind,
    pub duration: Option<f32>,
    pub progress: Option<f32>,
}

#[derive(Clone, Copy)]
pub enum BlockAnimationKind {
    Move,
    Rotate { pivot: IVec3, clockwise: bool },
    SpawnScale,
}

#[derive(Component)]
pub struct AnimatedBlock {
    from_translation: Vec3,
    to_translation: Vec3,
    from_rotation: Quat,
    to_rotation: Quat,
    from_scale: Vec3,
    to_scale: Vec3,
    kind: BlockAnimationKind,
    elapsed: f32,
    timing: AnimationTiming,
}

#[derive(Clone, Copy)]
pub struct PusherAnimation {
    pub duration: f32,
    pub from_extension: f32,
    pub to_extension: f32,
}

#[derive(Component)]
pub struct AnimatedPusher {
    base_translation: Vec3,
    direction: Vec3,
    elapsed: f32,
    duration: f32,
    from_extension: f32,
    to_extension: f32,
}

#[derive(Component)]
pub struct WeldSpark {
    velocity: Vec3,
    elapsed: f32,
    duration: f32,
}

impl AnimatedPusher {
    pub fn new(animation: PusherAnimation, base_translation: Vec3) -> Self {
        Self {
            base_translation,
            direction: Vec3::NEG_Z,
            elapsed: 0.0,
            duration: animation.duration,
            from_extension: animation.from_extension,
            to_extension: animation.to_extension,
        }
    }
}

impl AnimatedBlock {
    pub fn new(animation: BlockAnimation, timing: AnimationTiming) -> Self {
        let timing = AnimationTiming {
            duration: animation.duration.unwrap_or(timing.duration),
            easing: timing.easing,
        };
        Self {
            from_translation: grid_to_world(animation.from_pos),
            to_translation: grid_to_world(animation.to_pos),
            from_rotation: Quat::from_rotation_y(animation.from_facing.yaw()),
            to_rotation: Quat::from_rotation_y(animation.to_facing.yaw()),
            from_scale: match animation.kind {
                BlockAnimationKind::Move | BlockAnimationKind::Rotate { .. } => Vec3::ONE,
                BlockAnimationKind::SpawnScale => Vec3::ZERO,
            },
            to_scale: Vec3::ONE,
            kind: animation.kind,
            elapsed: animation.progress.unwrap_or(0.0).clamp(0.0, 1.0) * timing.duration,
            timing,
        }
    }
}

impl WeldSpark {
    pub fn new(velocity: Vec3, duration: f32) -> Self {
        Self {
            velocity,
            elapsed: 0.0,
            duration,
        }
    }
}

pub fn animate_blocks(
    time: Res<Time>,
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut Transform, &mut AnimatedBlock)>,
    mut pushers: Query<(Entity, &mut Transform, &mut AnimatedPusher), Without<AnimatedBlock>>,
    mut sparks: Query<
        (Entity, &mut Transform, &mut WeldSpark),
        (Without<AnimatedBlock>, Without<AnimatedPusher>),
    >,
) {
    for (entity, mut transform, mut animation) in &mut blocks {
        animation.elapsed += time.delta_secs();
        let t = (animation.elapsed / animation.timing.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let eased = match animation.timing.easing {
            AnimationEasing::Linear => t,
            AnimationEasing::SmoothStep => t * t * (3.0 - 2.0 * t),
        };
        transform.translation = match animation.kind {
            BlockAnimationKind::Rotate { pivot, clockwise } => {
                rotate_world_pos_y(animation.from_translation, pivot, clockwise, eased)
            }
            _ => animation.from_translation.lerp(animation.to_translation, eased),
        };
        transform.rotation = animation.from_rotation.slerp(animation.to_rotation, eased);
        transform.scale = animation.from_scale.lerp(animation.to_scale, eased);

        if t >= 1.0 {
            transform.translation = animation.to_translation;
            transform.rotation = animation.to_rotation;
            transform.scale = animation.to_scale;
            commands.entity(entity).remove::<AnimatedBlock>();
        }
    }

    for (entity, mut transform, mut animation) in &mut pushers {
        animation.elapsed += time.delta_secs();
        let t = (animation.elapsed / animation.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let extension = animation.from_extension.lerp(animation.to_extension, t);
        transform.translation = animation.base_translation + animation.direction * extension;

        if t >= 1.0 {
            transform.translation =
                animation.base_translation + animation.direction * animation.to_extension;
            commands.entity(entity).remove::<AnimatedPusher>();
        }
    }

    for (entity, mut transform, mut spark) in &mut sparks {
        spark.elapsed += time.delta_secs();
        let t = (spark.elapsed / spark.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        transform.translation += spark.velocity * time.delta_secs();
        transform.scale = Vec3::splat((1.0 - t).max(0.0));

        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn rotate_world_pos_y(from: Vec3, pivot: IVec3, clockwise: bool, t: f32) -> Vec3 {
    let pivot = grid_to_world(pivot);
    let rel = from - pivot;
    let angle = if clockwise {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    } * t;
    let rotation = Quat::from_rotation_y(angle);
    pivot + rotation.mul_vec3(rel)
}
