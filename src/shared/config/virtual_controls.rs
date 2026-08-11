/// 虚拟遥感单个控件相对固定锚点的偏移与缩放
///
/// `offset_*` / 基准尺寸以 [`VIRTUAL_LAYOUT_REF_EDGE`] 为参考短边存档；
/// 运行时用 `Val::VMin` 按视口短边比例显示（与 DPI / window.scale_factor 无关）。
/// 竖屏时短边是宽，控件会随短边变小。
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualControlTransform {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
}

/// 遥感布局存档参考短边（逻辑单位；`ref_px / 720 * 100%` → VMin）
pub const VIRTUAL_LAYOUT_REF_EDGE: f32 = 720.0;

impl VirtualControlTransform {
    pub const fn new(offset_x: f32, offset_y: f32, scale: f32) -> Self {
        Self {
            offset_x,
            offset_y,
            scale,
        }
    }

    pub const fn identity() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
}

/// 虚拟遥感全部控件布局（锚点固定，仅存 offset/scale）
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VirtualControlsLayout {
    pub joystick: VirtualControlTransform,
    pub jump: VirtualControlTransform,
    pub place: VirtualControlTransform,
    pub delete: VirtualControlTransform,
    pub pause: VirtualControlTransform,
    pub simulate: VirtualControlTransform,
    pub sim_pause: VirtualControlTransform,
    pub sim_fast: VirtualControlTransform,
    pub sim_step: VirtualControlTransform,
    pub rotate: VirtualControlTransform,
    pub alternate: VirtualControlTransform,
    pub block_config: VirtualControlTransform,
    #[serde(default = "default_virtual_inventory")]
    pub inventory: VirtualControlTransform,
    #[serde(default = "default_virtual_pick")]
    pub pick: VirtualControlTransform,
}

fn default_virtual_inventory() -> VirtualControlTransform {
    VirtualControlTransform::new(24.0, 282.0, 1.5)
}

/// 选取按钮的默认位置
fn default_virtual_pick() -> VirtualControlTransform {
    VirtualControlTransform::new(150.0, 282.0, 1.5)
}

impl VirtualControlsLayout {
    pub const DEFAULT: Self = Self {
        // 基于玩家手调布局校准：摇杆等距；右下三角同尺寸圆弧；右上/右侧齐边等距
        // 方向摇杆保持原尺寸；右下三键在旧尺寸上 ×1.2；其余按钮 ×1.5。
        joystick: VirtualControlTransform::new(75.1875, 75.1875, 1.8123217),
        jump: VirtualControlTransform::new(41.4327, 242.987, 1.4673486),
        place: VirtualControlTransform::new(174.2844, 160.9637, 1.4673486),
        delete: VirtualControlTransform::new(247.7921, 23.2175, 1.4673486),
        pause: VirtualControlTransform::new(24.0, 24.0, 1.5),
        simulate: VirtualControlTransform::new(110.0, 24.0, 1.5),
        sim_pause: VirtualControlTransform::new(110.0, 24.0, 1.5),
        sim_fast: VirtualControlTransform::new(196.0, 24.0, 1.5),
        sim_step: VirtualControlTransform::new(282.0, 24.0, 1.5),
        rotate: VirtualControlTransform::new(24.0, 196.0, 1.5),
        alternate: VirtualControlTransform::new(24.0, 110.0, 1.5),
        block_config: VirtualControlTransform::new(58.625, 268.75, 1.5),
        inventory: VirtualControlTransform::new(24.0, 282.0, 1.5),
        pick: VirtualControlTransform::new(150.0, 282.0, 1.5),
    };

    pub fn transform(&self, id: VirtualControlId) -> VirtualControlTransform {
        match id {
            VirtualControlId::Joystick => self.joystick,
            VirtualControlId::Jump => self.jump,
            VirtualControlId::Place => self.place,
            VirtualControlId::Delete => self.delete,
            VirtualControlId::Pause => self.pause,
            VirtualControlId::Simulate => self.simulate,
            VirtualControlId::SimPause => self.sim_pause,
            VirtualControlId::SimFast => self.sim_fast,
            VirtualControlId::SimStep => self.sim_step,
            VirtualControlId::Rotate => self.rotate,
            VirtualControlId::Alternate => self.alternate,
            VirtualControlId::BlockConfig => self.block_config,
            VirtualControlId::Inventory => self.inventory,
            VirtualControlId::Pick => self.pick,
        }
    }

    pub fn set_transform(&mut self, id: VirtualControlId, transform: VirtualControlTransform) {
        match id {
            VirtualControlId::Joystick => self.joystick = transform,
            VirtualControlId::Jump => self.jump = transform,
            VirtualControlId::Place => self.place = transform,
            VirtualControlId::Delete => self.delete = transform,
            VirtualControlId::Pause => self.pause = transform,
            VirtualControlId::Simulate => self.simulate = transform,
            VirtualControlId::SimPause => self.sim_pause = transform,
            VirtualControlId::SimFast => self.sim_fast = transform,
            VirtualControlId::SimStep => self.sim_step = transform,
            VirtualControlId::Rotate => self.rotate = transform,
            VirtualControlId::Alternate => self.alternate = transform,
            VirtualControlId::BlockConfig => self.block_config = transform,
            VirtualControlId::Inventory => self.inventory = transform,
            VirtualControlId::Pick => self.pick = transform,
        }
    }
}

/// 虚拟遥感控件标识（锚点种类固定）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VirtualControlId {
    Joystick,
    Jump,
    Place,
    Delete,
    Pause,
    Simulate,
    SimPause,
    SimFast,
    SimStep,
    Rotate,
    Alternate,
    BlockConfig,
    Inventory,
    Pick,
}

impl VirtualControlId {
    pub const ALL: &[Self] = &[
        Self::Joystick,
        Self::Jump,
        Self::Place,
        Self::Delete,
        Self::Pause,
        Self::Simulate,
        Self::SimPause,
        Self::SimFast,
        Self::SimStep,
        Self::Rotate,
        Self::Alternate,
        Self::BlockConfig,
        Self::Inventory,
        Self::Pick,
    ];

    pub fn anchor(self) -> VirtualControlAnchor {
        match self {
            Self::Joystick => VirtualControlAnchor::BottomLeft,
            Self::Jump | Self::Place | Self::Delete => VirtualControlAnchor::BottomRight,
            Self::Pause | Self::Simulate | Self::SimPause | Self::SimFast | Self::SimStep => {
                VirtualControlAnchor::TopRight
            }
            Self::Rotate | Self::Alternate | Self::Inventory => {
                VirtualControlAnchor::TopRightColumn
            }
            Self::BlockConfig => VirtualControlAnchor::BottomCenter,
            Self::Pick => VirtualControlAnchor::BottomCenter,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Joystick => "virtual.joystick",
            Self::Jump => "action.jump_or_fly_up",
            Self::Place => "action.place",
            Self::Delete => "action.delete",
            Self::Pause => "action.pause",
            Self::Simulate => "action.simulate",
            Self::SimPause => "virtual.sim_pause",
            Self::SimFast => "action.simulation_fast",
            Self::SimStep => "action.simulation_step",
            Self::Rotate => "action.rotate_or_rollback",
            Self::Alternate => "action.alternate",
            Self::BlockConfig => "virtual.block_config",
            Self::Inventory => "action.inventory",
            Self::Pick => "action.pick",
        }
    }
}

/// 虚拟控件屏幕锚点（不可在设置中更改）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VirtualControlAnchor {
    BottomLeft,
    BottomRight,
    TopRight,
    TopRightColumn,
    BottomCenter,
}
