use glam::IVec3;

use crate::blocks::LaserOpticsBehavior;
use crate::world::direction::Facing;

/// 按光学行为计算激光反射方向
pub fn reflect_laser(
    optics: LaserOpticsBehavior,
    block_facing: Facing,
    incoming: IVec3,
) -> Vec<IVec3> {
    match optics {
        LaserOpticsBehavior::Mirror => reflect_mirror(block_facing, incoming).into_iter().collect(),
        LaserOpticsBehavior::VerticalMirror => reflect_vertical_mirror(block_facing, incoming)
            .into_iter()
            .collect(),
        LaserOpticsBehavior::Splitter => reflect_splitter(block_facing, incoming),
    }
}

// 水平镜子：面片 000/101/111/010 再绕 Y +90°，按模型法线反射
fn reflect_mirror(block_facing: Facing, incoming: IVec3) -> Option<IVec3> {
    if incoming.y != 0 {
        return None;
    }
    match block_facing {
        // 模型对角在世界中呈 /：X↔Z 同号互换
        Facing::East | Facing::West => match incoming {
            IVec3::X => Some(IVec3::Z),
            IVec3::NEG_X => Some(IVec3::NEG_Z),
            IVec3::Z => Some(IVec3::X),
            IVec3::NEG_Z => Some(IVec3::NEG_X),
            _ => None,
        },
        // 模型对角在世界中呈 \：X↔Z 异号互换
        Facing::North | Facing::South => match incoming {
            IVec3::X => Some(IVec3::NEG_Z),
            IVec3::NEG_X => Some(IVec3::Z),
            IVec3::Z => Some(IVec3::NEG_X),
            IVec3::NEG_Z => Some(IVec3::X),
            _ => None,
        },
    }
}

// 垂直镜子：面片在局部 X-Y 对角，水平轴取局部 +X 经朝向旋转后的世界方向
fn reflect_vertical_mirror(block_facing: Facing, incoming: IVec3) -> Option<IVec3> {
    let axis = local_x_axis(block_facing);
    let neg_axis = -axis;
    match incoming {
        IVec3::Y => Some(neg_axis),
        IVec3::NEG_Y => Some(axis),
        dir if dir == axis => Some(IVec3::NEG_Y),
        dir if dir == neg_axis => Some(IVec3::Y),
        _ => None,
    }
}

// 分光镜：任一轴入射，另外两轴出射；朝向只决定正负（North 基准三元组：-X, +Y, +Z）
fn reflect_splitter(block_facing: Facing, incoming: IVec3) -> Vec<IVec3> {
    let triad = [
        rotate_by_facing(block_facing, IVec3::NEG_X),
        rotate_by_facing(block_facing, IVec3::Y),
        rotate_by_facing(block_facing, IVec3::Z),
    ];
    for i in 0..3 {
        let a = triad[i];
        let b = triad[(i + 1) % 3];
        let c = triad[(i + 2) % 3];
        if incoming == a {
            return vec![b, c];
        }
        if incoming == -a {
            return vec![-b, -c];
        }
    }
    Vec::new()
}

fn rotate_by_facing(facing: Facing, v: IVec3) -> IVec3 {
    match facing {
        Facing::North => v,
        Facing::East => IVec3::new(v.z, v.y, -v.x),
        Facing::South => IVec3::new(-v.x, v.y, -v.z),
        Facing::West => IVec3::new(-v.z, v.y, v.x),
    }
}

// 方块朝向旋转后，模型局部 +X 对应的世界轴
fn local_x_axis(facing: Facing) -> IVec3 {
    match facing {
        Facing::North => IVec3::X,
        Facing::East => IVec3::NEG_Z,
        Facing::South => IVec3::NEG_X,
        Facing::West => IVec3::Z,
    }
}

