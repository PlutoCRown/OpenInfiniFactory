//! 方块表现侧模型与连接器类型（模拟核心不依赖）

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
    Device { blocked_offset: IVec3 },
}

/// 方块模型网格枚举
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ModelMesh {
    ConveyorBase,
    ConveyorBelt,
    DrillBody,
    DrillTip,
    PusherBody,
    PusherHead,
    Large,
    Medium,
    Small,
    Plate,
    /// 竖立告示板（薄在 Z，大面朝 ±Z）
    SignBoard,
    RotatorBase,
    RotatorDisk,
    RotatorRing,
    RodX,
    RodY,
    RodZ,
    MirrorFace,
    VerticalMirrorFace,
    SplitterFace,
    SuctionCup,
}

/// 方块模型材质枚举
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ModelMaterial {
    ConveyorBase,
    ConveyorBelt,
    DrillTip,
    Frame,
    DarkFrame,
    Belt,
    BeltStripe,
    WeldCore,
    Welding,
    Wire,
    Signal,
    Power,
    DetectorBody,
    Pusher,
    Platform,
    PlatformBase,
    Wood,
    WoodTexture,
    BorderedWoodTexture,
    StoneTexture,
    Lift,
    Rotation,
    Drill,
    Laser,
    Mirror,
    System,
    SystemAccent,
    TeleportIn,
    TeleportOut,
    SuctionCup,
}

/// 方块模型零件
#[derive(Clone, Copy)]
pub struct BlockModelPart {
    pub mesh: ModelMesh,
    pub material: ModelMaterial,
    pub translation: [f32; 3],
    pub scale: [f32; 3],
    pub yaw_radians: f32,
}

impl BlockModelPart {
    pub const fn new(mesh: ModelMesh, material: ModelMaterial, translation: [f32; 3]) -> Self {
        Self {
            mesh,
            material,
            translation,
            scale: [1.0, 1.0, 1.0],
            yaw_radians: 0.0,
        }
    }

    pub const fn scaled(mut self, scale: [f32; 3]) -> Self {
        self.scale = scale;
        self
    }

    pub const fn yawed(mut self, yaw_radians: f32) -> Self {
        self.yaw_radians = yaw_radians;
        self
    }
}

/// 方块模型组装方式
#[derive(Clone, Copy)]
pub enum BlockModel {
    Default,
    Parts(&'static [BlockModelPart]),
    PartsOnly(&'static [BlockModelPart]),
    PusherParts(&'static [BlockModelPart]),
}
