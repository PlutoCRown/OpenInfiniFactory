use bevy::prelude::*;

use super::SelectionOverlayDefinition;

pub struct DeleteOverlay;

impl SelectionOverlayDefinition for DeleteOverlay {
    fn material() -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgba(1.0, 0.08, 0.04, 0.38),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }
    }
}
