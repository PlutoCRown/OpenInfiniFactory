//! 材料 / 印花 / 滚刷资源包：扫描目录、安装模拟 catalog，并提供表现侧注册表

mod load;
mod meta;
mod registry;

pub use load::{
    load_global_material_packs, merge_puzzle_material_packs, reload_global_only,
    MaterialPackRegistries,
};
pub use registry::{
    MaterialBlockPresentation, MaterialBlockRegistry, PaintMaterialPresentation,
    PaintMaterialRegistry, StampMaterialPresentation, StampMaterialRegistry,
};
