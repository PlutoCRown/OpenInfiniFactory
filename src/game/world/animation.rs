use crate::game::simulation::motion::{BlockMotion, BlockMotionKind, PusherMotion};
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

/// 放映用方块动画参数（可由 BlockMotion 转换；duration/progress 仅表现层填写）
#[derive(Clone, Copy)]
pub struct BlockAnimation {
    pub block_id: crate::game::blocks::BlockId,
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub from_facing: Facing,
    pub to_facing: Facing,
    pub kind: BlockAnimationKind,
    pub duration: Option<f32>,
    pub progress: Option<f32>,
}

/// 放映用方块动画种类
#[derive(Clone, Copy)]
pub enum BlockAnimationKind {
    Move,
    Rotate { pivot: IVec3, clockwise: bool },
    SpawnScale,
}

/// 将模拟侧 BlockMotionKind 转为放映侧种类
impl From<BlockMotionKind> for BlockAnimationKind {
    fn from(kind: BlockMotionKind) -> Self {
        match kind {
            BlockMotionKind::Move => Self::Move,
            BlockMotionKind::Rotate { pivot, clockwise } => Self::Rotate { pivot, clockwise },
            BlockMotionKind::SpawnScale => Self::SpawnScale,
        }
    }
}

/// 将模拟侧 BlockMotion 转为放映侧 BlockAnimation
impl From<BlockMotion> for BlockAnimation {
    fn from(motion: BlockMotion) -> Self {
        Self {
            block_id: motion.block_id,
            from_pos: motion.from_pos,
            to_pos: motion.to_pos,
            from_facing: motion.from_facing,
            to_facing: motion.to_facing,
            kind: motion.kind.into(),
            duration: None,
            progress: None,
        }
    }
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

/// 放映用推杆动画参数（可由 PusherMotion 转换；duration 仅表现层填写）
#[derive(Clone, Copy)]
pub struct PusherAnimation {
    /// 仅放映时填写；模拟输出保持 None
    pub duration: Option<f32>,
    pub from_extension: f32,
    pub to_extension: f32,
}

/// 将模拟侧 PusherMotion 转为放映侧 PusherAnimation
impl From<PusherMotion> for PusherAnimation {
    fn from(motion: PusherMotion) -> Self {
        Self {
            duration: None,
            from_extension: motion.from_extension,
            to_extension: motion.to_extension,
        }
    }
}

#[derive(Component)]
pub struct AnimatedPusher {
    base_translation: Vec3,
    direction: Vec3,
    /// Head=1.0，Stage=0.5
    extension_factor: f32,
    elapsed: f32,
    duration: f32,
    from_extension: f32,
    to_extension: f32,
}

#[derive(Component)]
pub struct AnimatedPusherRod {
    xy_scale: Vec3,
    elapsed: f32,
    duration: f32,
    from_extension: f32,
    to_extension: f32,
}

/// 钻头 Head：模拟激活后绕局部前进轴（Z）持续旋转
#[derive(Component, Default)]
pub struct SpinningDrillHead;

#[derive(Component)]
pub struct LaserBeamBurst {
    origin: Vec3,
    direction: Vec3,
    full_length: f32,
    axis_scale: f32,
    elapsed: f32,
    duration: f32,
}

impl LaserBeamBurst {
    pub fn new(
        origin: Vec3,
        direction: Vec3,
        full_length: f32,
        axis_scale: f32,
        duration: f32,
    ) -> Self {
        Self {
            origin,
            direction,
            full_length,
            axis_scale,
            elapsed: 0.0,
            duration,
        }
    }

    fn apply(&self, thickness: f32, transform: &mut Transform) {
        transform.translation = self.origin + self.direction * (self.full_length * 0.5);
        transform.scale = Vec3::new(thickness, thickness, self.axis_scale);
    }
}

#[derive(Component)]
pub struct WeldSpark {
    velocity: Vec3,
    elapsed: f32,
    duration: f32,
    /// 等本回合移动动画结束后再开始飞溅
    delay: f32,
}

/// 焊接成功：在焊点连线中点的法平面上向外扩散的 2D 粒子
#[derive(Component)]
pub struct WeldBurstParticle {
    origin: Vec3,
    /// 法平面内单位方向
    dir: Vec3,
    /// 终点半径（世界单位，约 1～2 格）
    target_radius: f32,
    elapsed: f32,
    duration: f32,
    delay: f32,
    size: f32,
}

/// MC 风格破坏碎片：重力落地 + 始终朝向镜头
#[derive(Component)]
pub struct BreakDebrisParticle {
    velocity: Vec3,
    elapsed: f32,
    lifetime: f32,
    ground_y: f32,
}

impl BreakDebrisParticle {
    pub fn new(velocity: Vec3, lifetime: f32, ground_y: f32) -> Self {
        Self {
            velocity,
            elapsed: 0.0,
            lifetime,
            ground_y,
        }
    }
}

impl AnimatedPusher {
    pub fn new(animation: PusherAnimation, base_translation: Vec3) -> Self {
        Self::with_factor(animation, base_translation, 1.0)
    }

    /// Stage 用 0.5、Head 用 1.0
    pub fn with_factor(
        animation: PusherAnimation,
        base_translation: Vec3,
        extension_factor: f32,
    ) -> Self {
        Self {
            base_translation,
            direction: Vec3::NEG_Z,
            extension_factor,
            elapsed: 0.0,
            duration: animation.duration.unwrap_or(0.0),
            from_extension: animation.from_extension,
            to_extension: animation.to_extension,
        }
    }
}

impl AnimatedPusherRod {
    pub fn new(animation: PusherAnimation, xy_scale: Vec3) -> Self {
        Self {
            xy_scale,
            elapsed: 0.0,
            duration: animation.duration.unwrap_or(0.0),
            from_extension: animation.from_extension,
            to_extension: animation.to_extension,
        }
    }

    fn apply(&self, extension: f32, transform: &mut Transform) {
        use crate::game::blocks::pusher::model::{
            ROD_BASE_LENGTH, pusher_rod_center_z, pusher_rod_length,
        };

        let length = pusher_rod_length(extension);
        transform.translation.z = pusher_rod_center_z(extension);
        transform.scale = Vec3::new(self.xy_scale.x, self.xy_scale.y, length / ROD_BASE_LENGTH);
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

    // 动画起始 Transform，用于复用实体时对齐到 from
    pub fn start_transform(&self) -> Transform {
        Transform {
            translation: self.from_translation,
            rotation: self.from_rotation,
            scale: self.from_scale,
            ..Default::default()
        }
    }
}

impl WeldSpark {
    pub fn new(velocity: Vec3, duration: f32, delay: f32) -> Self {
        Self {
            velocity,
            elapsed: 0.0,
            duration,
            delay: delay.max(0.0),
        }
    }
}

impl WeldBurstParticle {
    pub fn new(
        origin: Vec3,
        dir: Vec3,
        target_radius: f32,
        duration: f32,
        delay: f32,
        size: f32,
    ) -> Self {
        Self {
            origin,
            dir,
            target_radius,
            elapsed: 0.0,
            duration,
            delay: delay.max(0.0),
            size,
        }
    }
}

pub fn animate_blocks(
    time: Res<Time>,
    simulation: Res<crate::game::state::SimulationState>,
    camera: Query<&GlobalTransform, With<crate::game::cameras::GameplayCamera>>,
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut Transform, &mut AnimatedBlock)>,
    mut pushers: Query<
        (Entity, &mut Transform, &mut AnimatedPusher),
        (
            Without<AnimatedBlock>,
            Without<AnimatedPusherRod>,
            Without<SpinningDrillHead>,
            Without<BreakDebrisParticle>,
        ),
    >,
    mut pusher_rods: Query<
        (Entity, &mut Transform, &mut AnimatedPusherRod),
        (
            Without<AnimatedBlock>,
            Without<AnimatedPusher>,
            Without<SpinningDrillHead>,
            Without<BreakDebrisParticle>,
        ),
    >,
    mut drill_heads: Query<
        &mut Transform,
        (
            With<SpinningDrillHead>,
            Without<AnimatedBlock>,
            Without<AnimatedPusher>,
            Without<AnimatedPusherRod>,
            Without<WeldSpark>,
            Without<WeldBurstParticle>,
            Without<LaserBeamBurst>,
            Without<BreakDebrisParticle>,
        ),
    >,
    mut sparks: Query<
        (Entity, &mut Transform, &mut WeldSpark),
        (
            Without<AnimatedBlock>,
            Without<AnimatedPusher>,
            Without<AnimatedPusherRod>,
            Without<SpinningDrillHead>,
            Without<WeldBurstParticle>,
            Without<LaserBeamBurst>,
            Without<BreakDebrisParticle>,
        ),
    >,
    mut weld_bursts: Query<
        (Entity, &mut Transform, &mut WeldBurstParticle),
        (
            Without<AnimatedBlock>,
            Without<AnimatedPusher>,
            Without<AnimatedPusherRod>,
            Without<SpinningDrillHead>,
            Without<WeldSpark>,
            Without<LaserBeamBurst>,
            Without<BreakDebrisParticle>,
        ),
    >,
    mut debris: Query<
        (Entity, &mut Transform, &mut BreakDebrisParticle),
        (
            Without<AnimatedBlock>,
            Without<AnimatedPusher>,
            Without<AnimatedPusherRod>,
            Without<SpinningDrillHead>,
            Without<WeldSpark>,
            Without<WeldBurstParticle>,
            Without<LaserBeamBurst>,
        ),
    >,
    mut laser_beams: Query<
        (Entity, &mut Transform, &mut LaserBeamBurst),
        (
            Without<AnimatedBlock>,
            Without<AnimatedPusher>,
            Without<AnimatedPusherRod>,
            Without<SpinningDrillHead>,
            Without<WeldSpark>,
            Without<WeldBurstParticle>,
            Without<BreakDebrisParticle>,
        ),
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
            _ => animation
                .from_translation
                .lerp(animation.to_translation, eased),
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
        transform.translation = animation.base_translation
            + animation.direction * (extension * animation.extension_factor);

        if t >= 1.0 {
            transform.translation = animation.base_translation
                + animation.direction * (animation.to_extension * animation.extension_factor);
            commands.entity(entity).remove::<AnimatedPusher>();
        }
    }

    for (entity, mut transform, mut animation) in &mut pusher_rods {
        animation.elapsed += time.delta_secs();
        let t = (animation.elapsed / animation.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let extension = animation.from_extension.lerp(animation.to_extension, t);
        animation.apply(extension, &mut transform);

        if t >= 1.0 {
            animation.apply(animation.to_extension, &mut transform);
            commands.entity(entity).remove::<AnimatedPusherRod>();
        }
    }

    if simulation.is_active() {
        // 约每模拟回合转两圈
        let delta = time.delta_secs() * (std::f32::consts::TAU * 2.0 / SIMULATION_TURN_SECONDS);
        for mut transform in &mut drill_heads {
            transform.rotate_local_z(delta);
        }
    }

    for (entity, mut transform, mut spark) in &mut sparks {
        spark.elapsed += time.delta_secs();
        if spark.elapsed < spark.delay {
            transform.scale = Vec3::ZERO;
            continue;
        }
        let active = spark.elapsed - spark.delay;
        let t = (active / spark.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        transform.translation += spark.velocity * time.delta_secs();
        transform.scale = Vec3::splat((1.0 - t).max(0.0));

        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut transform, mut particle) in &mut weld_bursts {
        particle.elapsed += time.delta_secs();
        if particle.elapsed < particle.delay {
            transform.scale = Vec3::ZERO;
            continue;
        }
        let active = particle.elapsed - particle.delay;
        let t = (active / particle.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        // 先快后慢飞到各自的随机终点距离
        let eased = 1.0 - (1.0 - t) * (1.0 - t);
        let radius = particle.target_radius * eased;
        transform.translation = particle.origin + particle.dir * radius;
        let fade = (1.0 - t).max(0.0);
        transform.scale = Vec3::splat(particle.size * (0.55 + 0.45 * fade));

        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }

    let camera_pos = camera
        .iter()
        .next()
        .map(|t| t.translation())
        .unwrap_or(Vec3::ZERO);
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle) in &mut debris {
        particle.elapsed += dt;
        particle.velocity.y -= 18.0 * dt;
        transform.translation += particle.velocity * dt;
        if transform.translation.y < particle.ground_y {
            transform.translation.y = particle.ground_y;
            particle.velocity.y *= -0.35;
            particle.velocity.x *= 0.72;
            particle.velocity.z *= 0.72;
            if particle.velocity.y.abs() < 0.6 {
                particle.velocity = Vec3::ZERO;
            }
        }
        let to_cam = (camera_pos - transform.translation).normalize_or_zero();
        if to_cam != Vec3::ZERO {
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, to_cam);
        }
        let life_t = (particle.elapsed / particle.lifetime.max(f32::EPSILON)).clamp(0.0, 1.0);
        if life_t > 0.7 {
            transform.scale = Vec3::splat(((1.0 - life_t) / 0.3).max(0.0));
        }
        if life_t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut transform, mut beam) in &mut laser_beams {
        beam.elapsed += time.delta_secs();
        let t = (beam.elapsed / beam.duration.max(f32::EPSILON)).clamp(0.0, 1.0);
        let thickness = (1.0 - t).max(0.0);
        beam.apply(thickness, &mut transform);

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
