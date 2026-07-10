use bevy::prelude::*;

use crate::game::blocks::BlockKind;
use crate::game::world::direction::Facing;

pub fn reflect_laser(kind: BlockKind, block_facing: Facing, incoming: IVec3) -> Vec<IVec3> {
    match kind {
        BlockKind::Mirror => reflect_mirror(block_facing, incoming).into_iter().collect(),
        BlockKind::VerticalMirror => reflect_vertical_mirror(block_facing, incoming)
            .into_iter()
            .collect(),
        BlockKind::Splitter => reflect_splitter(block_facing, incoming),
        _ => Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_reflects_using_model_geometry() {
        assert_eq!(
            reflect_laser(BlockKind::Mirror, Facing::North, IVec3::X),
            vec![IVec3::NEG_Z]
        );
        assert_eq!(
            reflect_laser(BlockKind::Mirror, Facing::East, IVec3::X),
            vec![IVec3::Z]
        );
    }

    #[test]
    fn vertical_mirror_reflects_using_local_x_axis() {
        // North：局部 +X 仍是 +X，与 Y 互换（法线取向使 +X → -Y）
        assert_eq!(
            reflect_laser(BlockKind::VerticalMirror, Facing::North, IVec3::Y),
            vec![IVec3::NEG_X]
        );
        assert_eq!(
            reflect_laser(BlockKind::VerticalMirror, Facing::North, IVec3::X),
            vec![IVec3::NEG_Y]
        );
        // East：局部 +X 变为 -Z；南向激光 +Z 应反射为 +Y
        assert_eq!(
            reflect_laser(BlockKind::VerticalMirror, Facing::East, IVec3::Z),
            vec![IVec3::Y]
        );
    }

    #[test]
    fn splitter_splits_to_the_other_two_axes() {
        // 朝北 +Z 入射 → +Y 与 -X（599 上方与 499）
        assert_eq!(
            reflect_laser(BlockKind::Splitter, Facing::North, IVec3::Z),
            vec![IVec3::NEG_X, IVec3::Y]
        );
        // 朝东 +Z 入射 → +Y 与 +X
        assert_eq!(
            reflect_laser(BlockKind::Splitter, Facing::East, IVec3::Z),
            vec![IVec3::Y, IVec3::X]
        );
        // 反向入射时两路出射一并取反
        assert_eq!(
            reflect_laser(BlockKind::Splitter, Facing::North, IVec3::NEG_Z),
            vec![IVec3::X, IVec3::NEG_Y]
        );
    }

    #[test]
    fn main_beam_stops_at_mirror_and_spawns_reflected_beam() {
        use crate::game::blocks::{BlockData, BlockKind};
        use crate::game::simulation::behaviors::trace_laser_for_test;
        use crate::game::world::grid::WorldBlocks;

        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(1, 0, 0),
            BlockData {
                kind: BlockKind::Laser,
                facing: Facing::East,
            },
        );
        world.insert(
            IVec3::new(2, 0, 0),
            BlockData {
                kind: BlockKind::Mirror,
                facing: Facing::North,
            },
        );
        world.insert(
            IVec3::new(5, 0, 0),
            BlockData {
                kind: BlockKind::Platform,
                facing: Facing::North,
            },
        );

        let mut beams = Vec::new();
        trace_laser_for_test(&mut world, IVec3::new(1, 0, 0), IVec3::X, 30, &mut beams, 0);

        assert!(
            beams.iter().any(|beam| {
                beam.pos == IVec3::new(1, 0, 0) && beam.direction == IVec3::X && beam.range == 1
            }),
            "main beam should stop at the mirror"
        );
        assert!(
            beams
                .iter()
                .any(|beam| { beam.pos == IVec3::new(2, 0, 0) && beam.direction == IVec3::NEG_Z }),
            "reflected beam should start from the mirror"
        );
        assert!(
            !beams
                .iter()
                .any(|beam| beam.pos == IVec3::new(1, 0, 0) && beam.range > 1),
            "main beam must not pass through the mirror"
        );
    }

    #[test]
    fn trace_laser_reflects_off_mirror_and_destroys_material() {
        use crate::game::blocks::{BlockData, BlockKind, MaterialKind};
        use crate::game::simulation::behaviors::trace_laser_for_test;
        use crate::game::world::grid::WorldBlocks;

        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(1, 0, 0),
            BlockData {
                kind: BlockKind::Laser,
                facing: Facing::East,
            },
        );
        world.insert(
            IVec3::new(2, 0, 0),
            BlockData {
                kind: BlockKind::Mirror,
                facing: Facing::North,
            },
        );
        let material_pos = IVec3::new(2, 0, -1);
        world.insert(
            material_pos,
            BlockData {
                kind: BlockKind::material_block_kind(MaterialKind::Basic).unwrap(),
                facing: Facing::North,
            },
        );

        let mut beams = Vec::new();
        trace_laser_for_test(&mut world, IVec3::new(1, 0, 0), IVec3::X, 30, &mut beams, 0);

        assert!(
            !world.is_material_at(material_pos),
            "reflected laser should destroy material on the mirror path"
        );
        assert!(beams.len() >= 2);
    }

    #[test]
    fn east_vertical_mirror_reflects_south_laser_up() {
        use crate::game::blocks::{BlockData, BlockKind};
        use crate::game::simulation::behaviors::trace_laser_for_test;
        use crate::game::world::grid::WorldBlocks;

        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::new(5, 9, 3),
            BlockData {
                kind: BlockKind::Laser,
                facing: Facing::South,
            },
        );
        world.insert(
            IVec3::new(5, 9, 9),
            BlockData {
                kind: BlockKind::VerticalMirror,
                facing: Facing::East,
            },
        );

        let mut beams = Vec::new();
        trace_laser_for_test(&mut world, IVec3::new(5, 9, 3), IVec3::Z, 30, &mut beams, 0);

        assert!(
            beams
                .iter()
                .any(|beam| { beam.pos == IVec3::new(5, 9, 9) && beam.direction == IVec3::Y }),
            "south laser on east vertical mirror should reflect up"
        );
    }
}
