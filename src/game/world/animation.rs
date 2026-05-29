use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::world::direction::Facing;
use crate::game::world::grid::grid_to_world;
use crate::game::world::rendering::BlockEntity;

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
}

#[derive(Component)]
pub struct AnimatedBlock {
    from_translation: Vec3,
    to_translation: Vec3,
    from_rotation: Quat,
    to_rotation: Quat,
    elapsed: f32,
    timing: AnimationTiming,
}

impl AnimatedBlock {
    pub fn new(animation: BlockAnimation, timing: AnimationTiming) -> Self {
        Self {
            from_translation: grid_to_world(animation.from_pos),
            to_translation: grid_to_world(animation.to_pos),
            from_rotation: Quat::from_rotation_y(animation.from_facing.yaw()),
            to_rotation: Quat::from_rotation_y(animation.to_facing.yaw()),
            elapsed: 0.0,
            timing,
        }
    }
}

pub fn animate_blocks(
    time: Res<Time>,
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut Transform, &mut AnimatedBlock), With<BlockEntity>>,
) {
    for (entity, mut transform, mut animation) in &mut blocks {
        animation.elapsed += time.delta_seconds();
        let t = (animation.elapsed / animation.timing.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let eased = match animation.timing.easing {
            AnimationEasing::Linear => t,
            AnimationEasing::SmoothStep => t * t * (3.0 - 2.0 * t),
        };
        transform.translation = animation
            .from_translation
            .lerp(animation.to_translation, eased);
        transform.rotation = animation.from_rotation.slerp(animation.to_rotation, eased);

        if t >= 1.0 {
            transform.translation = animation.to_translation;
            transform.rotation = animation.to_rotation;
            commands.entity(entity).remove::<AnimatedBlock>();
        }
    }
}

pub fn pair_block_animations(
    before: &HashMap<IVec3, (crate::game::world::blocks::BlockKind, Facing)>,
    after: &HashMap<IVec3, (crate::game::world::blocks::BlockKind, Facing)>,
) -> HashMap<IVec3, BlockAnimation> {
    let mut animations = HashMap::new();
    let mut used_before = std::collections::HashSet::new();

    for (to_pos, (kind, to_facing)) in after {
        let mut best = None;
        let mut best_distance = i32::MAX;

        for (from_pos, (from_kind, from_facing)) in before {
            if used_before.contains(from_pos) || from_kind != kind {
                continue;
            }

            let distance = (*to_pos - *from_pos).abs().element_sum();
            if distance < best_distance {
                best = Some((*from_pos, *from_facing));
                best_distance = distance;
            }
        }

        let Some((from_pos, from_facing)) = best else {
            continue;
        };

        used_before.insert(from_pos);
        if from_pos != *to_pos || from_facing != *to_facing {
            animations.insert(
                *to_pos,
                BlockAnimation {
                    from_pos,
                    to_pos: *to_pos,
                    from_facing,
                    to_facing: *to_facing,
                },
            );
        }
    }

    animations
}
