use super::traits::BlockMeta;
use super::{BlockDefinition, BlockKind, ColorSpec, MaterialKind};

/// 材料块共用的元数据形状
pub trait BasicBlockDef {
    const KIND: BlockKind;
    const MATERIAL: MaterialKind;
    const NAME_KEY: &'static str;
    const SHORT_NAME_KEY: &'static str;
    const DESCRIPTION_KEY: &'static str;
    const COLOR: ColorSpec;
}

impl<T: BasicBlockDef + Send + Sync> BlockMeta for T {
    fn id(&self) -> BlockKind {
        T::KIND
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::material(
            self.id(),
            T::NAME_KEY,
            T::SHORT_NAME_KEY,
            T::DESCRIPTION_KEY,
            T::COLOR,
        )
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        Some(T::MATERIAL)
    }
}
