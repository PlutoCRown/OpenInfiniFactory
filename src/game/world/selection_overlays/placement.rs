use bevy::prelude::*;

use super::SelectionOverlayDefinition;

pub struct PlacementOverlay;

impl SelectionOverlayDefinition for PlacementOverlay {
    fn material() -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgba(0.7, 0.95, 1.0, 0.34),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.92,
            reflectance: 0.0,
            ..default()
        }
    }
}
