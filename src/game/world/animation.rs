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
    elapsed: f32,
    timing: AnimationTiming,
}

#[derive(Clone, Copy)]
pub struct PistonAnimation {
    pub direction: IVec3,
    pub duration: f32,
}

#[derive(Component)]
pub struct AnimatedPiston {
    direction: Vec3,
    elapsed: f32,
    duration: f32,
}

#[derive(Component)]
pub struct WeldSpark {
    velocity: Vec3,
    elapsed: f32,
    duration: f32,
}

impl AnimatedPiston {
    pub fn new(animation: PistonAnimation) -> Self {
        Self {
            direction: animation.direction.as_vec3(),
            elapsed: 0.0,
            duration: animation.duration,
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
                BlockAnimationKind::Move => Vec3::ONE,
                BlockAnimationKind::SpawnScale => Vec3::ZERO,
            },
            to_scale: Vec3::ONE,
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
    mut pistons: Query<(Entity, &mut Transform, &mut AnimatedPiston), Without<AnimatedBlock>>,
    mut sparks: Query<
        (Entity, &mut Transform, &mut WeldSpark),
        (Without<AnimatedBlock>, Without<AnimatedPiston>),
    >,
) {
    for (entity, mut transform, mut animation) in &mut blocks {
        animation.elapsed += time.delta_secs();
        let t = (animation.elapsed / animation.timing.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let eased = match animation.timing.easing {
            AnimationEasing::Linear => t,
            AnimationEasing::SmoothStep => t * t * (3.0 - 2.0 * t),
        };
        transform.translation = animation
            .from_translation
            .lerp(animation.to_translation, eased);
        transform.rotation = animation.from_rotation.slerp(animation.to_rotation, eased);
        transform.scale = animation.from_scale.lerp(animation.to_scale, eased);

        if t >= 1.0 {
            transform.translation = animation.to_translation;
            transform.rotation = animation.to_rotation;
            transform.scale = animation.to_scale;
            commands.entity(entity).remove::<AnimatedBlock>();
        }
    }

    for (entity, mut transform, mut animation) in &mut pistons {
        animation.elapsed += time.delta_secs();
        let t = (animation.elapsed / animation.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let extension = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        transform.translation = animation.direction * extension * 0.20;

        if t >= 1.0 {
            transform.translation = Vec3::ZERO;
            commands.entity(entity).remove::<AnimatedPiston>();
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
