use bevy::prelude::Image;

use crate::game::world::procedural_textures::{from_fn, platform_pixel};

pub fn image() -> Image {
    from_fn(platform_pixel)
}
