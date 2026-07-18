//! 材料面能力：有向 / 脆弱 / 印花 / Connectable

use glam::IVec3;

use crate::world::direction::Facing;

use super::material_catalog::MaterialBlockDef;

/// 材料静态属性（由 catalog 定义派生）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaterialProps {
    /// 是否四向有向（facing 参与玩法与存档）
    pub directional: bool,
    /// 运动冲突时碎裂而非阻挡
    pub fragile: bool,
    /// 印花材料：面片模型、不可焊接
    pub is_stamp: bool,
    /// 局部六面可否焊接/滚刷/印花：+X -X +Y -Y +Z -Z（相对 facing=North 的局部系）
    pub connectable: [bool; 6],
}

impl MaterialProps {
    pub const DEFAULT: Self = Self {
        directional: false,
        fragile: false,
        is_stamp: false,
        connectable: [true; 6],
    };

    pub const STAMP: Self = Self {
        // 朝向由附着面法线表达，不走方块 yaw（否则面片会双重旋转悬空）
        directional: false,
        fragile: false,
        is_stamp: true,
        connectable: [false; 6],
    };

    pub const FRAGILE: Self = Self {
        directional: false,
        fragile: true,
        is_stamp: false,
        connectable: [true; 6],
    };

    /// 从材料方块定义派生属性（非印花）
    pub fn from_material_def(def: &MaterialBlockDef) -> Self {
        Self {
            directional: def.directional,
            fragile: def.fragile,
            is_stamp: false,
            connectable: def.connectable,
        }
    }

    /// 印花材料属性（不可焊接；朝向由附着面表达）
    pub fn stamp_props(fragile: bool) -> Self {
        Self {
            directional: false,
            fragile,
            is_stamp: true,
            connectable: [false; 6],
        }
    }
}

/// 单位法线 → Connectable 下标；非单位轴返回 None
pub fn local_face_index(local_normal: IVec3) -> Option<usize> {
    match local_normal {
        IVec3 { x: 1, y: 0, z: 0 } => Some(0),
        IVec3 { x: -1, y: 0, z: 0 } => Some(1),
        IVec3 { x: 0, y: 1, z: 0 } => Some(2),
        IVec3 { x: 0, y: -1, z: 0 } => Some(3),
        IVec3 { x: 0, y: 0, z: 1 } => Some(4),
        IVec3 { x: 0, y: 0, z: -1 } => Some(5),
        _ => None,
    }
}

/// 世界法线在给定 facing 下是否 Connectable
pub fn material_face_connectable(
    props: MaterialProps,
    facing: Facing,
    world_normal: IVec3,
) -> bool {
    let local = facing.inverse_rotate_offset(world_normal);
    local_face_index(local).is_some_and(|index| props.connectable[index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec3;

    #[test]
    fn default_materials_are_fully_connectable() {
        let props = MaterialProps::DEFAULT;
        assert!(!props.fragile);
        assert!(!props.is_stamp);
        assert!(!props.directional);
        assert!(material_face_connectable(props, Facing::North, IVec3::Y));
        assert!(material_face_connectable(props, Facing::East, IVec3::X));
    }

    #[test]
    fn stamp_props_block_all_faces() {
        let props = MaterialProps::stamp_props(false);
        assert!(props.is_stamp);
        assert!(!props.directional);
        assert!(!props.fragile);
        assert!(!material_face_connectable(
            props,
            Facing::North,
            IVec3::NEG_Z
        ));
    }

    #[test]
    fn fragile_constant_matches_glass() {
        let props = MaterialProps::FRAGILE;
        assert!(props.fragile);
        assert!(!props.is_stamp);
        assert!(!props.directional);
    }

    #[test]
    fn stamp_constant_matches_stamp_props() {
        assert_eq!(MaterialProps::STAMP, MaterialProps::stamp_props(false));
    }
}
