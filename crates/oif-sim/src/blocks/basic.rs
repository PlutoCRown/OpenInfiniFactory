use super::traits::BlockMeta;
use super::{BlockDefinition, BlockKind, ColorSpec, MaterialKind};

#[derive(Clone, Copy)]
pub enum BasicBlockLayer {
    Scene,
    Material(MaterialKind),
}

/// 场景块与材料块共用的元数据形状
pub trait BasicBlockDef {
    const KIND: BlockKind;
    const LAYER: BasicBlockLayer;
    const NAME_KEY: &'static str;
    const SHORT_NAME_KEY: &'static str;
    const COLOR: ColorSpec;
}

impl<T: BasicBlockDef + Send + Sync> BlockMeta for T {
    fn id(&self) -> BlockKind {
        T::KIND
    }

    fn definition(&self) -> BlockDefinition {
        match T::LAYER {
            BasicBlockLayer::Scene => {
                BlockDefinition::scene(self.id(), T::NAME_KEY, T::SHORT_NAME_KEY, T::COLOR)
            }
            BasicBlockLayer::Material(_) => {
                BlockDefinition::material(self.id(), T::NAME_KEY, T::SHORT_NAME_KEY, T::COLOR)
            }
        }
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        match T::LAYER {
            BasicBlockLayer::Scene => None,
            BasicBlockLayer::Material(material) => Some(material),
        }
    }
}
