use bevy::prelude::*;

use super::SelectionOverlayDefinition;

pub struct ActiveFactoryDebugOverlay;

impl SelectionOverlayDefinition for ActiveFactoryDebugOverlay {
    fn material() -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgb(0.12, 0.90, 0.22),
            unlit: true,
            ..default()
        }
    }
}

pub struct InactiveFactoryDebugOverlay;

impl SelectionOverlayDefinition for InactiveFactoryDebugOverlay {
    fn material() -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgb(0.95, 0.12, 0.08),
            unlit: true,
            ..default()
        }
    }
}

pub struct MaterialDebugOverlay;

impl SelectionOverlayDefinition for MaterialDebugOverlay {
    fn material() -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgb(0.12, 0.50, 1.0),
            unlit: true,
            ..default()
        }
    }
}
