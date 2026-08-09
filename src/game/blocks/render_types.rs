//! 方块表现侧类型（模拟核心不依赖）

use bevy::prelude::IVec3;

/// 渲染侧连接器提示
#[derive(Clone, Copy, Default)]
pub struct RenderBehavior {
    pub weld_connector: Option<WeldConnectorBehavior>,
    pub wire_connector: Option<WireConnectorBehavior>,
}

/// 朝向设备的导线连接渲染：指定面不接线
pub fn render_directional_wire_device(blocked_offset: IVec3) -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::Device { blocked_offset }),
        ..Default::default()
    }
}

/// 焊接连接器渲染
#[derive(Clone, Copy)]
pub enum WeldConnectorBehavior {
    AllSides,
    Offset(IVec3),
}

/// 导线连接器渲染
#[derive(Clone, Copy)]
pub enum WireConnectorBehavior {
    Wire,
    /// 除该面外可接
    Device { blocked_offset: IVec3 },
    /// 仅该面可接
    AllowOnly { offset: IVec3 },
}

/// 仅底面接线的用电器（旋转器、抬升器）
pub fn render_bottom_wire_device() -> RenderBehavior {
    RenderBehavior {
        wire_connector: Some(WireConnectorBehavior::AllowOnly {
            offset: IVec3::NEG_Y,
        }),
        ..Default::default()
    }
}

/// 告示牌零件网格
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum SignMesh {
    Board,
    Pole,
}
