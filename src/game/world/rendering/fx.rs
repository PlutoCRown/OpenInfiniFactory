use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::simulation::{BreakDebris, LaserBeam, LaserBeamStop};
use crate::game::world::animation::{
    BreakDebrisParticle, LaserBeamBurst, WeldBurstParticle, WeldSpark,
};
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

/// 激光等非焊接行为的立方火花（焊接已改用 `spawn_weld_bursts`）
pub fn spawn_weld_sparks(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    positions: &[IVec3],
    delay: f32,
) {
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
                Transform::from_translation(origin + offset).with_scale(Vec3::ZERO),
                WeldSpark::new(velocity, 0.28, delay),
            ));
        }
    }
}

/// 焊接成功：两焊点连线中点，沿法平面向外扩散的 2D 白粒子（黄泛光）
pub fn spawn_weld_bursts(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    pairs: &[(IVec3, IVec3)],
    delay: f32,
) {
    const COUNT: i32 = 42;
    /// 相对初版 1.85 的三倍飞溅速度
    const SPEED: f32 = 5.55;
    const SIZE: f32 = 0.11;

    for &(a, b) in pairs {
        let wa = grid_to_world(a);
        let wb = grid_to_world(b);
        let origin = (wa + wb) * 0.5;
        let axis = (wb - wa).normalize_or_zero();
        if axis == Vec3::ZERO {
            continue;
        }
        let tangent = if axis.cross(Vec3::Y).length_squared() > 0.05 {
            axis.cross(Vec3::Y).normalize()
        } else {
            axis.cross(Vec3::X).normalize()
        };
        let bitangent = axis.cross(tangent);
        // 方片躺在法平面上（局部 XY ↔ 法平面，Z 对齐焊点连线）
        let rotation = Quat::from_rotation_arc(Vec3::Z, axis);

        for i in 0..COUNT {
            let angle = std::f32::consts::TAU * (i as f32) / (COUNT as f32)
                + (i as f32 * 0.37).sin() * 0.12;
            let dir = (tangent * angle.cos() + bitangent * angle.sin()).normalize_or_zero();
            // 终点距离在 1～2 格之间伪随机
            let target_radius = 1.0 + ((i as f32 * 2.13).sin() * 0.5 + 0.5);
            let duration = target_radius / SPEED;
            commands.spawn((
                Mesh3d(assets.weld_burst_quad.clone()),
                MeshMaterial3d(assets.weld_burst_material.clone()),
                Transform::from_translation(origin)
                    .with_rotation(rotation)
                    .with_scale(Vec3::ZERO),
                WeldBurstParticle::new(origin, dir, target_radius, duration, delay, SIZE),
            ));
        }
    }
}

/// MC 风格：采样材料贴图随机 UV 方片，公告板朝镜头，重力落地
pub fn spawn_break_debris(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    debris: &[BreakDebris],
) {
    const COUNT: i32 = 16;
    for item in debris {
        let origin = grid_to_world(item.pos);
        let material = assets.break_debris_material(item.kind);
        let ground_y = 0.02_f32;
        let seed = (item.pos.x.wrapping_mul(73856093)
            ^ item.pos.y.wrapping_mul(19349663)
            ^ item.pos.z.wrapping_mul(83492791)) as u32;
        for i in 0..COUNT {
            let n = seed.wrapping_add((i as u32).wrapping_mul(2654435761));
            let u_cell = (n % 4) as f32;
            let v_cell = ((n / 4) % 4) as f32;
            let u0 = u_cell * 0.25;
            let v0 = v_cell * 0.25;
            let u1 = u0 + 0.25;
            let v1 = v0 + 0.25;
            let fx = ((n >> 3) & 255) as f32 / 255.0;
            let fy = ((n >> 11) & 255) as f32 / 255.0;
            let fz = ((n >> 19) & 255) as f32 / 255.0;
            let offset = Vec3::new(fx - 0.5, fy * 0.6 + 0.1, fz - 0.5) * 0.85;
            let velocity = Vec3::new((fx - 0.5) * 3.2, 2.4 + fy * 2.8, (fz - 0.5) * 3.2);
            let size = 0.10 + ((n >> 7) & 7) as f32 * 0.012;
            let mesh = meshes.add({
                let h = size * 0.5;
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![[-h, -h, 0.0], [h, -h, 0.0], [h, h, 0.0], [-h, h, 0.0]],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    vec![
                        [0.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                    ],
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_UV_0,
                    vec![[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
                )
                .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]))
            });
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(origin + offset),
                BreakDebrisParticle::new(velocity, 1.1, ground_y),
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
                WeldSpark::new(velocity, 0.56, 0.0),
            ));
        }
    }
}
