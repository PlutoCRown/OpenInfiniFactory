//! 场景方块资源包：扫描 meta.json / model.glb，安装模拟 catalog，并提供表现侧注册表

mod glb;
mod load;
mod meta;
mod registry;

#[cfg(all(feature = "native-tools", not(target_arch = "wasm32")))]
pub mod bake_icons;

pub use glb::{load_scene_glb, SceneGltfHandles};
pub use load::{load_global_scene_blocks, merge_puzzle_scene_blocks, reload_global_only};
pub use registry::{SceneBlockPresentation, SceneBlockRegistry};

/// 从磁盘加载预烘焙 icon.png
pub fn load_icon_png(
    path: &std::path::Path,
    images: &mut bevy::prelude::Assets<bevy::prelude::Image>,
) -> Option<bevy::prelude::Handle<bevy::prelude::Image>> {
    use bevy::asset::RenderAssetUsages;
    use bevy::prelude::*;

    let bytes = std::fs::read(path).ok()?;
    let mut image = Image::from_buffer(
        &bytes,
        bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .ok()?;
    image.sampler =
        bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor::linear());
    Some(images.add(image))
}
