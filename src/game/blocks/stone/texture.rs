use bevy::prelude::Image;

use crate::game::world::procedural_textures::{from_fn, material_pixel};

pub fn image() -> Image {
    from_fn(|x, y| material_pixel(x, y, [124, 128, 132], 89))
}
