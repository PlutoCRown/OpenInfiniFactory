use super::{rgb, Block, BlockDefinition, BlockKind, BlockTexture, EditableBlock, MaterialKind};

pub trait BasicBlockDef: Send + Sync {
    const KIND: BlockKind;
    const LAYER: BasicBlockLayer;
    const NAME_KEY: &'static str;
    const SHORT_NAME_KEY: &'static str;
    const COLOR: super::ColorSpec;
    const SLOT_COLOR: super::ColorSpec;
    const TEXTURE: BlockTexture;
}

#[derive(Clone, Copy)]
pub enum BasicBlockLayer {
    Scene,
    Material(MaterialKind),
}

pub struct RegisteredBasicBlock<T>(std::marker::PhantomData<T>);

impl<T> RegisteredBasicBlock<T> {
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: BasicBlockDef> Block for RegisteredBasicBlock<T> {
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

pub struct BasicBlockRegistration {
    pub kind: BlockKind,
    pub block: &'static (dyn Block + Send + Sync),
    pub editable_block: Option<&'static (dyn EditableBlock + Send + Sync)>,
    pub editable: bool,
}

inventory::collect!(BasicBlockRegistration);

pub struct Grass;

impl<T: BasicBlockDef> EditableBlock for RegisteredBasicBlock<T> {}

impl BasicBlockDef for Grass {
    const KIND: BlockKind = BlockKind::Grass;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.grass";
    const SHORT_NAME_KEY: &'static str = "short.grass";
    const COLOR: super::ColorSpec = rgb(0.34, 0.62, 0.24);
    const SLOT_COLOR: super::ColorSpec = rgb(0.27, 0.56, 0.20);
    const TEXTURE: BlockTexture = BlockTexture::Grass;
}

pub struct Stone;

impl BasicBlockDef for Stone {
    const KIND: BlockKind = BlockKind::Stone;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.stone";
    const SHORT_NAME_KEY: &'static str = "short.stone";
    const COLOR: super::ColorSpec = rgb(0.43, 0.43, 0.42);
    const SLOT_COLOR: super::ColorSpec = rgb(0.42, 0.42, 0.40);
    const TEXTURE: BlockTexture = BlockTexture::Stone;
}

pub struct Dirt;

impl BasicBlockDef for Dirt {
    const KIND: BlockKind = BlockKind::Dirt;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.dirt";
    const SHORT_NAME_KEY: &'static str = "short.dirt";
    const COLOR: super::ColorSpec = rgb(0.40, 0.27, 0.16);
    const SLOT_COLOR: super::ColorSpec = rgb(0.42, 0.26, 0.14);
    const TEXTURE: BlockTexture = BlockTexture::Dirt;
}

pub struct Planks;

impl BasicBlockDef for Planks {
    const KIND: BlockKind = BlockKind::Planks;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.planks";
    const SHORT_NAME_KEY: &'static str = "short.planks";
    const COLOR: super::ColorSpec = rgb(0.66, 0.45, 0.25);
    const SLOT_COLOR: super::ColorSpec = rgb(0.62, 0.40, 0.20);
    const TEXTURE: BlockTexture = BlockTexture::Wood;
}

pub struct BasicMaterial;

impl BasicBlockDef for BasicMaterial {
    const KIND: BlockKind = BlockKind::Material;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Basic);
    const NAME_KEY: &'static str = "block.material";
    const SHORT_NAME_KEY: &'static str = "short.material";
    const COLOR: super::ColorSpec = rgb(0.82, 0.82, 0.86);
    const SLOT_COLOR: super::ColorSpec = rgb(0.74, 0.74, 0.78);
    const TEXTURE: BlockTexture = BlockTexture::Material;
}

pub struct IronMaterial;

impl BasicBlockDef for IronMaterial {
    const KIND: BlockKind = BlockKind::IronMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Iron);
    const NAME_KEY: &'static str = "block.iron_material";
    const SHORT_NAME_KEY: &'static str = "short.iron_material";
    const COLOR: super::ColorSpec = rgb(0.62, 0.64, 0.66);
    const SLOT_COLOR: super::ColorSpec = rgb(0.54, 0.56, 0.58);
    const TEXTURE: BlockTexture = BlockTexture::IronMaterial;
}

pub struct CopperMaterial;

impl BasicBlockDef for CopperMaterial {
    const KIND: BlockKind = BlockKind::CopperMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Copper);
    const NAME_KEY: &'static str = "block.copper_material";
    const SHORT_NAME_KEY: &'static str = "short.copper_material";
    const COLOR: super::ColorSpec = rgb(0.78, 0.42, 0.22);
    const SLOT_COLOR: super::ColorSpec = rgb(0.68, 0.34, 0.16);
    const TEXTURE: BlockTexture = BlockTexture::CopperMaterial;
}

macro_rules! register_basic_block {
    ($static_name:ident: $ty:ty, editable: $editable:expr) => {
        pub static $static_name: RegisteredBasicBlock<$ty> = RegisteredBasicBlock::new();

        inventory::submit! {
            BasicBlockRegistration {
                kind: <$ty as BasicBlockDef>::KIND,
                block: &$static_name,
                editable_block: if $editable { Some(&$static_name) } else { None },
                editable: $editable,
            }
        }
    };
}

register_basic_block!(GRASS: Grass, editable: true);
register_basic_block!(STONE: Stone, editable: true);
register_basic_block!(DIRT: Dirt, editable: true);
register_basic_block!(PLANKS: Planks, editable: true);
register_basic_block!(MATERIAL: BasicMaterial, editable: false);
register_basic_block!(IRON_MATERIAL: IronMaterial, editable: false);
register_basic_block!(COPPER_MATERIAL: CopperMaterial, editable: false);
