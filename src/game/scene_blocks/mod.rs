//! 场景方块资源包：扫描 meta.json / model.glb 或 texture.png，安装模拟 catalog，并提供表现侧注册表

mod glb;
mod load;
mod meta;
mod registry;

#[cfg(all(feature = "native-tools", not(target_arch = "wasm32")))]
pub mod bake_icons;

pub use glb::{load_collision_triangles, load_scene_glb, SceneGltfHandles};
pub use load::{load_global_scene_blocks, merge_puzzle_scene_blocks, reload_global_only};
pub use registry::{SceneBlockPresentation, SceneBlockRegistry};

/// 从磁盘加载预烘焙 icon.png（UI 用线性采样）
pub fn load_icon_png(
    path: &std::path::Path,
    images: &mut bevy::prelude::Assets<bevy::prelude::Image>,
) -> Option<bevy::prelude::Handle<bevy::prelude::Image>> {
    load_png_with_sampler(
        path,
        images,
        bevy::image::ImageSamplerDescriptor::linear(),
        true,
    )
}

/// 方块外观 texture.png：像素锐利 + 可重复（sRGB）
pub fn load_block_texture_png(
    path: &std::path::Path,
    images: &mut bevy::prelude::Assets<bevy::prelude::Image>,
) -> Option<bevy::prelude::Handle<bevy::prelude::Image>> {
    load_png_with_sampler(path, images, block_nearest_sampler("block_texture"), true)
}

/// 方块法线 normal.png：线性色域 + 像素锐利
pub fn load_block_normal_png(
    path: &std::path::Path,
    images: &mut bevy::prelude::Assets<bevy::prelude::Image>,
) -> Option<bevy::prelude::Handle<bevy::prelude::Image>> {
    load_png_with_sampler(path, images, block_nearest_sampler("block_normal"), false)
}

fn block_nearest_sampler(label: &str) -> bevy::image::ImageSamplerDescriptor {
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};

    ImageSamplerDescriptor {
        label: Some(label.into()),
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..ImageSamplerDescriptor::default()
    }
}

fn load_png_with_sampler(
    path: &std::path::Path,
    images: &mut bevy::prelude::Assets<bevy::prelude::Image>,
    sampler: bevy::image::ImageSamplerDescriptor,
    is_srgb: bool,
) -> Option<bevy::prelude::Handle<bevy::prelude::Image>> {
    use bevy::asset::RenderAssetUsages;
    use bevy::prelude::*;

    let bytes = std::fs::read(path).ok()?;
    let mut image = Image::from_buffer(
        &bytes,
        bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
        bevy::image::CompressedImageFormats::NONE,
        is_srgb,
        bevy::image::ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .ok()?;
    image.sampler = bevy::image::ImageSampler::Descriptor(sampler);
    Some(images.add(image))
}
