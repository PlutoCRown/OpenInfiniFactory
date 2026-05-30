use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone, Copy)]
pub enum ProceduralTexture {
    Material,
    IronMaterial,
    CopperMaterial,
}

pub fn block_texture(kind: ProceduralTexture) -> Image {
    const SIZE: u32 = 32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let [r, g, b] = match kind {
                ProceduralTexture::Material => material_pixel(x, y, [210, 188, 118], 131),
                ProceduralTexture::IronMaterial => material_pixel(x, y, [158, 166, 170], 149),
                ProceduralTexture::CopperMaterial => material_pixel(x, y, [201, 112, 58], 167),
            };
            data.extend_from_slice(&[r, g, b, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

fn material_pixel(x: u32, y: u32, base: [u8; 3], seed: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, seed);
    let fleck = ((x * 11 + y * 17 + noise as u32) % 29) < 3;
    let band = (x + y + seed) % 9 == 0;
    if fleck {
        shade(base, noise, 38)
    } else if band {
        shade(base, noise, -22)
    } else {
        shade(base, noise, 18)
    }
}

fn texture_noise(x: u32, y: u32, seed: u32) -> u8 {
    let mut value = x
        .wrapping_mul(73_856_093)
        .wrapping_add(y.wrapping_mul(19_349_663))
        .wrapping_add(seed.wrapping_mul(83_492_791));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0xff) as u8
}

fn shade(base: [u8; 3], noise: u8, amount: i16) -> [u8; 3] {
    let delta = (noise as i16 - 128) * amount / 128;
    [
        (base[0] as i16 + delta).clamp(0, 255) as u8,
        (base[1] as i16 + delta).clamp(0, 255) as u8,
        (base[2] as i16 + delta).clamp(0, 255) as u8,
    ]
}
