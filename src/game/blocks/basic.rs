use bevy::prelude::{Color, Image};

use crate::game::state::UiPanelId;

use super::traits::{BlockMeta, BlockRender, BlockUi, PlaceableBlock};
use super::{BlockDefinition, BlockKind, MaterialKind};
use crate::game::blocks::ColorSpec;

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
    const ITEM_SLOT_COLOR: ColorSpec;

    fn block_texture() -> Option<Image> {
        None
    }
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

impl<T: BasicBlockDef + Send + Sync> BlockRender for T {
    fn block_texture(&self) -> Option<Image> {
        T::block_texture()
    }
}

impl<T: BasicBlockDef + Send + Sync> PlaceableBlock for T {
    fn item_slot_color(&self) -> Color {
        T::ITEM_SLOT_COLOR.color()
    }
}

impl<T: BasicBlockDef + Send + Sync> BlockUi for T {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}
