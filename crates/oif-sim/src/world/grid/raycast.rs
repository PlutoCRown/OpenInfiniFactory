//! 网格坐标转换与射线检测

use glam::{IVec3, Vec3};

use super::{REACH, WorldBlocks};

/// 射线命中的格子与法线
#[derive(Clone, Copy)]
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

/// 起始格某一侧面（相对玩家为近或远）
#[derive(Clone, Copy, Debug)]
pub struct EditDragFace {
    pub axis: usize,
    /// 该轴世界坐标上的平面位置（格的 min 或 max）
    pub plane: f32,
    /// 相对射线原点是否为远侧面
    pub far: bool,
}

/// 射线与某个近/远侧面的正向交点
#[derive(Clone, Copy, Debug)]
pub struct EditDragFaceHit {
    pub t: f32,
    pub axis: usize,
    pub point: Vec3,
    pub far: bool,
}

/// 起始格六侧面（3 近 + 3 远）及其与射线的交点
#[derive(Clone, Debug)]
pub struct EditDragFaces {
    pub faces: [EditDragFace; 6],
    pub hits: Vec<EditDragFaceHit>,
}

/// 相对玩家所在卦限：取起始格 3 近侧面 + 3 远侧面，并求所有正向交点
pub fn edit_drag_faces(origin: Vec3, dir: Vec3, start: IVec3) -> EditDragFaces {
    let dir = dir.normalize_or_zero();
    let mut faces = [EditDragFace {
        axis: 0,
        plane: 0.0,
        far: false,
    }; 6];
    let mut hits = Vec::with_capacity(6);
    if dir == Vec3::ZERO {
        return EditDragFaces { faces, hits };
    }

    for axis in 0..3 {
        let min = start[axis] as f32;
        let max = min + 1.0;
        let (near_plane, far_plane) = if (origin[axis] - min).abs() <= (origin[axis] - max).abs() {
            (min, max)
        } else {
            (max, min)
        };
        faces[axis * 2] = EditDragFace {
            axis,
            plane: near_plane,
            far: false,
        };
        faces[axis * 2 + 1] = EditDragFace {
            axis,
            plane: far_plane,
            far: true,
        };

        let d = dir[axis];
        if d.abs() < 1e-6 {
            continue;
        }
        for (plane, far) in [(near_plane, false), (far_plane, true)] {
            let t = (plane - origin[axis]) / d;
            if t < 1e-6 || t > REACH {
                continue;
            }
            hits.push(EditDragFaceHit {
                t,
                axis,
                point: origin + dir * t,
                far,
            });
        }
    }
    EditDragFaces { faces, hits }
}

/// 编辑拖拽框选：线/面模式落到格子（近/远侧面求交，远侧面带距离惩罚）
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

    let hits = edit_drag_faces(origin, dir, start).hits;
    if hits.is_empty() {
        return None;
    }

    // 焦点优先级：score = t - (远侧面 ? 5 : 0)，取最大；远交点须比近交点再远 5 格才夺权
    let hit = *hits
        .iter()
        .max_by(|a, b| {
            let score = |h: &EditDragFaceHit| {
                if h.far {
                    h.t - FAR_FACE_PRIORITY_PENALTY
                } else {
                    h.t
                }
            };
            score(a)
                .partial_cmp(&score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    Some(match mode {
        EditSelectionMode::Plane => snap_plane_on_normal(hit.point, start, plane_normal),
        EditSelectionMode::Line => {
            let mut raw = world_to_grid(hit.point);
            raw[hit.axis] = start[hit.axis];
            let delta = raw - start;
            if delta == IVec3::ZERO {
                start
            } else {
                snap_line_on_plane(hit.point, start, strongest_axis_vec(delta))
            }
        }
        EditSelectionMode::Point => unreachable!(),
    })
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
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max, REACH) {
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
        if let Some((distance, normal)) = ray_aabb(origin, dir, min, max, REACH) {
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
) -> Option<(f32, IVec3)> {
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
        Some((t_enter.max(0.0), normal))
    }
}

fn axis_vec(axis: usize) -> IVec3 {
    match axis {
        0 => IVec3::X,
        1 => IVec3::Y,
        _ => IVec3::Z,
    }
}
