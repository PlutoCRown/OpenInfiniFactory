use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::{BlockBehavior, BlockMeta};
use crate::blocks::{BlockDefinition, BlockKind, MaterialKind, rgb};

/// 印花材料：占格附着薄面片（有碰撞占体积），不可焊接
pub struct StampMaterial;

pub static BLOCK: BlockImpl<StampMaterial> = BlockImpl(StampMaterial);

impl BlockMeta for StampMaterial {
    fn id(&self) -> BlockKind {
        BlockKind::StampMaterial
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::material(
            self.id(),
            "block.stamp_material",
            "short.stamp_material",
            "desc.stamp_material",
            rgb(0.95, 0.12, 0.10),
        )
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Stamp)
    }
}

impl BlockBehavior for StampMaterial {}

register_block!(BLOCK, BlockKind::StampMaterial, editable: false);
