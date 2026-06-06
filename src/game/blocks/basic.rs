use crate::game::state::UiPanelId;

use super::{BlockDefinition, BlockKind, BlockTexture, MaterialKind};
use super::traits::{BlockMeta, BlockUi};
use crate::game::blocks::{rgb, ColorSpec};

#[derive(Clone, Copy)]
pub enum BasicBlockLayer {
    Scene,
    Material(MaterialKind),
}

/// Scene and material blocks share the same metadata shape.
pub trait BasicBlockDef {
    const KIND: BlockKind;
    const LAYER: BasicBlockLayer;
    const NAME_KEY: &'static str;
    const SHORT_NAME_KEY: &'static str;
    const COLOR: ColorSpec;
    const SLOT_COLOR: ColorSpec;
    const TEXTURE: BlockTexture;
}

impl<T: BasicBlockDef + Send + Sync> BlockMeta for T {
    fn id(&self) -> BlockKind {
        T::KIND
    }

    fn definition(&self) -> BlockDefinition {
        match T::LAYER {
            BasicBlockLayer::Scene => BlockDefinition::scene(
                self.id(),
                T::NAME_KEY,
                T::SHORT_NAME_KEY,
                T::COLOR,
                T::SLOT_COLOR,
            )
            .textured(T::TEXTURE),
            BasicBlockLayer::Material(_) => BlockDefinition::material(
                self.id(),
                T::NAME_KEY,
                T::SHORT_NAME_KEY,
                T::COLOR,
                T::SLOT_COLOR,
            )
            .textured(T::TEXTURE),
        }
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        match T::LAYER {
            BasicBlockLayer::Scene => None,
            BasicBlockLayer::Material(material) => Some(material),
        }
    }
}

impl<T: BasicBlockDef + Send + Sync> BlockUi for T {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}
