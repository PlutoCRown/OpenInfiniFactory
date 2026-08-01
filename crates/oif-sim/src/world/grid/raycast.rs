//! 网格坐标转换与射线检测

use glam::{IVec3, Vec3};

use super::{REACH, WorldBlocks};

/// 射线命中的格子与法线
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetHit {
    pub pos: IVec3,
    pub normal: IVec3,
}

/// 射线与无限平面求交（距离不超过 REACH）
pub fn raycast_infinite_plane(
    origin: Vec3,
    dir: Vec3,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> Option<Vec3> {
    let normal = plane_normal.normalize_or_zero();
    if normal == Vec3::ZERO {
        return None;
    }
    let denom = dir.dot(normal);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (plane_point - origin).dot(normal) / denom;
    if t < 0.0 || t > REACH {
        return None;
    }
    Some(origin + dir * t)
}

/// 世界坐标转格子坐标（向下取整）
pub fn world_to_grid(pos: Vec3) -> IVec3 {
    pos.floor().as_ivec3()
}

/// 编辑框选模式（与配置层同构，避免 oif-sim 依赖主 crate）
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EditSelectionMode {
    #[default]
    Point,
    Line,
    Plane,
}

/// 远侧面焦点优先级惩罚（世界单位 / 格）：远交点须再远这么多才能压过近交点
pub const FAR_FACE_PRIORITY_PENALTY: f32 = 5.0;

/// 编辑拖拽框选：线/面模式落到格子（起始格 min/max 面求交，远面带距离惩罚）
pub fn raycast_edit_drag_grid(
    origin: Vec3,
    dir: Vec3,
    start: IVec3,
    mode: EditSelectionMode,
    plane_normal: IVec3,
) -> Option<IVec3> {
    if mode == EditSelectionMode::Point {
        return None;
    }
    let dir = dir.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    // 起始格虚拟立方体：挡住射线穿透，穿出之后的平面交点一律丢掉
    let t_occlude = ray_aabb(
        origin,
        dir,
        start.as_vec3(),
        start.as_vec3() + Vec3::ONE,
        REACH,
    )
    .map(|(_, t_exit, _)| t_exit)
    .unwrap_or(REACH);

    // 每轴对格的 min/max 面求交；与玩家分居格心两侧的为远面（score = t - 5）
    let mut best: Option<(f32, Vec3, usize)> = None;
    for axis in 0..3 {
        let d = dir[axis];
        if d.abs() < 1e-6 {
            continue;
        }
        let min = start[axis] as f32;
        let center = min + 0.5;
        // 附着法线指向外侧时，丢掉朝宿主内侧的那张面（如西面放置不交东侧面）
        let skip_plane = if plane_normal[axis] > 0 {
            Some(min)
        } else if plane_normal[axis] < 0 {
            Some(min + 1.0)
        } else {
            None
        };
        for plane in [min, min + 1.0] {
            if skip_plane == Some(plane) {
                continue;
            }
            let t = (plane - origin[axis]) / d;
            if !(1e-6..REACH).contains(&t) || t > t_occlude + 1e-4 {
                continue;
            }
            let far = (plane - center) * (origin[axis] - center) < 0.0;
            let score = if far {
                t - FAR_FACE_PRIORITY_PENALTY
            } else {
                t
            };
            if best.map_or(true, |(best_score, _, _)| score > best_score) {
                best = Some((score, origin + dir * t, axis));
            }
        }
    }
    let (_, point, axis) = best?;

    let mut end = match mode {
        EditSelectionMode::Plane => snap_plane_on_normal(point, start, plane_normal),
        EditSelectionMode::Line => {
            let mut raw = world_to_grid(point);
            raw[axis] = start[axis];
            let delta = raw - start;
            if delta == IVec3::ZERO {
                start
            } else {
                snap_line_on_plane(point, start, strongest_axis_vec(delta))
            }
        }
        EditSelectionMode::Point => unreachable!(),
    };
    // 终点不得沿法线反方向越过起点（线选沿轴拉回宿主时钳回）
    for axis in 0..3 {
        let n = plane_normal[axis];
        if n != 0 && (end[axis] - start[axis]) * n < 0 {
            end[axis] = start[axis];
        }
    }
    Some(end)
}

fn snap_plane_on_normal(hit: Vec3, start: IVec3, normal: IVec3) -> IVec3 {
    let grid = world_to_grid(hit);
    if normal.x.abs() != 0 {
        IVec3::new(start.x, grid.y, grid.z)
    } else if normal.y.abs() != 0 {
        IVec3::new(grid.x, start.y, grid.z)
    } else {
        IVec3::new(grid.x, grid.y, start.z)
    }
}

fn strongest_axis_vec(delta: IVec3) -> IVec3 {
    if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        IVec3::X
    } else if delta.y.abs() >= delta.z.abs() {
        IVec3::Y
    } else {
        IVec3::Z
    }
}

fn snap_line_on_plane(hit: Vec3, start: IVec3, axis: IVec3) -> IVec3 {
    let grid = world_to_grid(hit);
    if axis.x != 0 {
        IVec3::new(grid.x, start.y, start.z)
    } else if axis.y != 0 {
        IVec3::new(start.x, grid.y, start.z)
    } else {
        IVec3::new(start.x, start.y, grid.z)
    }
}

/// 格子坐标转世界中心点
pub fn grid_to_world(pos: IVec3) -> Vec3 {
    pos.as_vec3() + Vec3::splat(0.5)
}

/// 对 blocks / system_blocks 做 AABB 射线，取最近命中
pub fn raycast_blocks(origin: Vec3, dir: Vec3, world: &WorldBlocks) -> Option<TargetHit> {
    let dir = dir.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let mut best: Option<(f32, TargetHit)> = None;

    // 无碰撞方块（如面片草）仍可被瞄准删除/取块；玩家物理碰撞另走 has_collision
    for (pos, block) in &world.blocks {
        if block.kind.is_generated_marker() {
            continue;
        }

        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, _, normal)) = ray_aabb(origin, dir, min, max, REACH) {
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }
    for (pos, block) in &world.system_blocks {
        if block.kind.is_generated_marker() {
            continue;
        }
        let center = grid_to_world(*pos);
        let min = center - Vec3::splat(0.5);
        let max = center + Vec3::splat(0.5);
        if let Some((distance, _, normal)) = ray_aabb(origin, dir, min, max, REACH) {
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, TargetHit { pos: *pos, normal }));
            }
        }
    }

    best.map(|(_, hit)| hit)
}

fn ray_aabb(
    origin: Vec3,
    dir: Vec3,
    min: Vec3,
    max: Vec3,
    max_distance: f32,
) -> Option<(f32, f32, IVec3)> {
    let mut t_enter = 0.0;
    let mut t_exit = max_distance;
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
        let mut near = (min_axis - origin_axis) * inv_dir;
        let mut far = (max_axis - origin_axis) * inv_dir;
        let near_normal = if inv_dir >= 0.0 {
            -axis_vec(axis)
        } else {
            axis_vec(axis)
        };

        if near > far {
            std::mem::swap(&mut near, &mut far);
        }

        if near > t_enter {
            t_enter = near;
            normal = near_normal;
        }
        t_exit = t_exit.min(far);
        if t_enter > t_exit {
            return None;
        }
    }

    if t_exit < 0.0 {
        None
    } else {
        Some((t_enter.max(0.0), t_exit, normal))
    }
}

fn axis_vec(axis: usize) -> IVec3 {
    match axis {
        0 => IVec3::X,
        1 => IVec3::Y,
        _ => IVec3::Z,
    }
}
