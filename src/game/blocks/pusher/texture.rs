use bevy::prelude::Image;

use crate::game::world::procedural_textures::{bordered_wood_pixel, from_fn};

pub fn bordered_wood() -> Image {
    from_fn(bordered_wood_pixel)
}
