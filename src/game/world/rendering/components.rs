use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::blocks::BlockKind;
use crate::game::simulation::structure_state::StructureKind;
use crate::game::world::grid::grid_to_world;

/// 根据方块位置与法线，计算瞄准面高亮的 Transform（贴在格面，不外浮；靠 depth_bias 叠层）
pub fn block_face_highlight_transform(block_pos: IVec3, normal: IVec3) -> Transform {
    let normal = normal.as_vec3().normalize();
    Transform {
        translation: grid_to_world(block_pos) + normal * 0.5,
        rotation: Quat::from_rotation_arc(Vec3::Y, normal),
        scale: Vec3::ONE,
    }
}

/// 世界方块实体所属渲染层
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockEntityLayer {
    /// 工厂/材料：可做移动动画，有 BlockId
    Animatable,
    /// 系统/虚拟：可与工厂材料重叠，独立实体
    System,
    /// 场景：无 ID、无动画
    Scene,
}

impl BlockEntityLayer {
    /// 按方块种类归入对应渲染层
    pub fn from_kind(kind: crate::game::blocks::BlockKind) -> Self {
        if kind.is_factory() || kind.is_material() {
            Self::Animatable
        } else if kind.is_system_layer() {
            Self::System
        } else {
            Self::Scene
        }
    }
}

/// 场景中已生成的方块实体标记
#[derive(Component)]
pub struct BlockEntity {
    pub pos: IVec3,
    pub id: crate::game::blocks::BlockId,
    pub layer: BlockEntityLayer,
}

/// 工厂活动调试半透明叠层
#[derive(Component)]
pub(super) struct FactoryDebugOverlay;

/// 游玩场景根标记（灯光、准星等）
#[derive(Component)]
pub struct GameplayScene;

/// 鼠标悬停方块线框
#[derive(Component)]
pub struct HoverMarker;

/// 瞄准面高亮平面
#[derive(Component)]
pub struct AimFaceHighlight;

/// 当前悬停结构的包围盒资源
#[derive(Resource, Default, Clone, Copy)]
pub struct HoverStructureBounds {
    pub bounds: Option<StructureBounds>,
}

/// 结构轴对齐包围盒
#[derive(Clone, Copy)]
pub struct StructureBounds {
    pub kind: StructureKind,
    pub min: IVec3,
    pub max: IVec3,
}

/// 放置预览方块标记
#[derive(Component)]
pub struct PlacementPreview;

/// 编辑操作预览标记
#[derive(Component)]
pub struct EditPreview;

/// 待生成方块的半透明预览标记
#[derive(Component)]
pub struct PendingGeneratedPreview;

/// 方块种类对应的离屏图标贴图
#[derive(Resource, Default)]
pub struct BlockIconAssets {
    pub(super) icons: HashMap<BlockKind, Handle<Image>>,
    /// 选区工具图标（非 BlockKind）
    pub(super) selection: Option<Handle<Image>>,
    /// 滚刷漆用 texture 作为选择格图标
    pub(super) paints: HashMap<crate::game::blocks::PaintMaterialId, Handle<Image>>,
}

impl BlockIconAssets {
    /// 取某种方块的图标句柄
    pub fn get(&self, kind: BlockKind) -> Option<Handle<Image>> {
        self.icons.get(&kind).cloned()
    }

    /// 选区工具图标
    pub fn selection(&self) -> Option<Handle<Image>> {
        self.selection.clone()
    }

    /// 滚刷漆图标
    pub fn paint(&self, id: crate::game::blocks::PaintMaterialId) -> Option<Handle<Image>> {
        self.paints.get(&id).cloned()
    }
}

/// 图标离屏渲染实体标记
#[derive(Component)]
pub(crate) struct BlockIconRenderEntity;

/// 图标离屏场景根（渲染结束后可整体销毁）
#[derive(Component)]
pub struct BlockIconRenderRoot;

/// 图标离屏相机
#[derive(Component)]
pub struct BlockIconRenderCamera;

/// 图标渲染剩余帧数，到 0 后停相机并清理
#[derive(Resource)]
pub struct BlockIconRenderState {
    pub(super) frames_remaining: u8,
}
