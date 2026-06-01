use bevy::prelude::*;

use super::SelectionOverlayDefinition;

pub struct AreaSelectionOverlay;

impl SelectionOverlayDefinition for AreaSelectionOverlay {
    fn material() -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgba(0.25, 0.95, 0.88, 0.34),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }
    }
}
