use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::ImageSampler;

#[derive(Clone, Copy)]
pub enum ProceduralTexture {
    Grass,
    Stone,
    Dirt,
    Planks,
    Glass,
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
                ProceduralTexture::Grass => grass_pixel(x, y),
                ProceduralTexture::Stone => stone_pixel(x, y),
                ProceduralTexture::Dirt => dirt_pixel(x, y),
                ProceduralTexture::Planks => planks_pixel(x, y),
                ProceduralTexture::Glass => glass_pixel(x, y),
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
    image.sampler = ImageSampler::nearest();
    image
}

fn grass_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 17);
    if y < 7 {
        let blade = ((x * 5 + y * 11 + noise as u32) % 13) < 4;
        if blade {
            shade([66, 135, 42], noise, 24)
        } else {
            shade([82, 154, 48], noise, 18)
        }
    } else {
        let root = ((x + y * 3 + noise as u32) % 17) < 3;
        if root {
            shade([78, 48, 26], noise, 18)
        } else {
            shade([118, 79, 43], noise, 22)
        }
    }
}

fn stone_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 41);
    let crack = ((x * 3 + y * 7 + noise as u32) % 23) == 0 || (x + y * 2) % 29 == 0;
    if crack {
        shade([70, 70, 68], noise, 10)
    } else {
        shade([122, 123, 119], noise, 26)
    }
}

fn dirt_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 73);
    let pebble = ((x * 13 + y * 5 + noise as u32) % 19) < 2;
    if pebble {
        shade([96, 73, 52], noise, 16)
    } else {
        shade([111, 72, 39], noise, 24)
    }
}

fn planks_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 109);
    let seam = y % 8 == 0 || x % 16 == 0;
    let grain = ((x * 7 + noise as u32) % 11) < 3;
    if seam {
        shade([86, 52, 25], noise, 10)
    } else if grain {
        shade([154, 104, 55], noise, 18)
    } else {
        shade([178, 121, 65], noise, 20)
    }
}

fn glass_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 127);
    let edge = x < 3 || y < 3 || x > 28 || y > 28;
    let glint = (x + y * 2 + noise as u32) % 31 == 0 || x == y;
    if edge {
        shade([126, 205, 226], noise, 16)
    } else if glint {
        shade([218, 250, 255], noise, 10)
    } else {
        shade([112, 184, 208], noise, 18)
    }
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
