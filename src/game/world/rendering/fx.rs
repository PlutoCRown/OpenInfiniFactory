use bevy::prelude::*;

use crate::game::simulation::{LaserBeam, LaserBeamStop};
use crate::game::world::animation::{LaserBeamBurst, WeldSpark};
use crate::game::world::grid::grid_to_world;
use crate::game::world::render_assets::WorldRenderAssets;

/// 连接器/激光杆默认长度（用于按实际距离缩放）
const CONNECTOR_BEAM_LENGTH: f32 = 0.55;

/// 由激光束数据算出世界空间起终点
fn laser_beam_segment(beam: &LaserBeam) -> (Vec3, Vec3) {
    let dir = beam.direction.as_vec3();
    let hit = beam.pos + beam.direction * beam.range;
    let start = if beam.emits_from_center {
        grid_to_world(beam.pos)
    } else {
        grid_to_world(beam.pos) + dir * 0.5
    };
    let end = match beam.stop {
        LaserBeamStop::Mirror => grid_to_world(hit),
        LaserBeamStop::Solid => grid_to_world(hit) - dir * 0.5,
        LaserBeamStop::Open => grid_to_world(hit) + dir * 0.5,
    };
    (start, end)
}

/// 生成激光束可视化实体
pub fn spawn_laser_beams(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    beams: &[LaserBeam],
    duration: f32,
) {
    for beam in beams {
        let direction = beam.direction.as_vec3();
        let (start, end) = laser_beam_segment(beam);
        let full_length = (end - start).length().max(0.01);
        let axis_scale = full_length / CONNECTOR_BEAM_LENGTH;
        // 统一用 Z 向杆再旋转到发射方向；按轴向选 X/Y mesh 会导致竖直激光几乎看不见
        let rotation = Quat::from_rotation_arc(Vec3::Z, direction);
        let transform = Transform::from_translation(start + (end - start) * 0.5)
            .with_rotation(rotation)
            .with_scale(Vec3::new(1.0, 1.0, axis_scale));

        commands.spawn((
            Mesh3d(assets.connector_mesh(IVec3::Z)),
            MeshMaterial3d(assets.laser_beam_material.clone()),
            transform,
            LaserBeamBurst::new(start, direction, full_length, axis_scale, duration),
        ));
    }
}

/// 在焊接位置生成火花粒子
pub fn spawn_weld_sparks(commands: &mut Commands, assets: &WorldRenderAssets, positions: &[IVec3]) {
    const VELOCITIES: [Vec3; 6] = [
        Vec3::new(1.60, 2.70, 0.42),
        Vec3::new(-1.44, 2.46, 0.76),
        Vec3::new(0.50, 2.86, -1.50),
        Vec3::new(-0.66, 2.28, -1.26),
        Vec3::new(1.18, 1.92, 1.34),
        Vec3::new(-1.26, 2.10, -0.34),
    ];

    for pos in positions {
        let origin = grid_to_world(*pos);
        for (index, velocity) in VELOCITIES.into_iter().enumerate() {
            let offset = Vec3::new(
                (index as f32 * 1.37).sin() * 0.20,
                0.04,
                (index as f32 * 2.11).cos() * 0.20,
            );
            commands.spawn((
                Mesh3d(assets.weld_spark.clone()),
                MeshMaterial3d(assets.laser_beam_material.clone()),
                Transform::from_translation(origin + offset),
                WeldSpark::new(velocity, 0.28),
            ));
        }
    }
}

/// 在验收成功位置生成火花粒子
pub fn spawn_acceptance_sparks(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    positions: &[IVec3],
) {
    const VELOCITIES: [Vec3; 12] = [
        Vec3::new(3.20, 5.40, 0.84),
        Vec3::new(-2.88, 4.92, 1.52),
        Vec3::new(1.00, 5.72, -3.00),
        Vec3::new(-1.32, 4.56, -2.52),
        Vec3::new(2.36, 3.84, 2.68),
        Vec3::new(-2.52, 4.20, -0.68),
        Vec3::new(2.84, 4.68, 1.18),
        Vec3::new(-1.08, 5.10, 2.44),
        Vec3::new(0.72, 3.36, -2.86),
        Vec3::new(-2.10, 5.52, 0.96),
        Vec3::new(1.94, 4.44, -1.74),
        Vec3::new(-0.84, 3.72, 2.96),
    ];

    for pos in positions {
        let origin = grid_to_world(*pos);
        for (index, velocity) in VELOCITIES.into_iter().enumerate() {
            let offset = Vec3::new(
                (index as f32 * 1.37).sin() * 0.40,
                0.08,
                (index as f32 * 2.11).cos() * 0.40,
            );
            commands.spawn((
                Mesh3d(assets.weld_spark.clone()),
                MeshMaterial3d(assets.acceptance_spark_material.clone()),
                Transform::from_translation(origin + offset),
                WeldSpark::new(velocity, 0.56),
            ));
        }
    }
}
