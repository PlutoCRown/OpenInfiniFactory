use bevy::prelude::*;

pub const BUTTON_BG: Color = Color::srgb(0.56, 0.56, 0.56);
pub const BUTTON_HOVER_BG: Color = Color::srgb(0.68, 0.68, 0.68);
pub const BUTTON_PRESSED_BG: Color = Color::srgb(0.40, 0.40, 0.40);
pub const BUTTON_LIGHT_EDGE: Color = Color::srgb(0.89, 0.89, 0.89);
pub const BUTTON_DARK_EDGE: Color = Color::srgb(0.02, 0.02, 0.02);

#[derive(Component, Clone, Default)]
pub struct HoverButton;

pub fn styled_button(
    style: Node,
    border: impl Into<BorderColor>,
    background: Color,
) -> impl Bundle {
    (
        Button,
        HoverButton,
        style,
        border.into(),
        BackgroundColor(background),
    )
}

pub fn raised_border() -> BorderColor {
    BorderColor {
        top: BUTTON_LIGHT_EDGE,
        left: BUTTON_LIGHT_EDGE,
        right: BUTTON_DARK_EDGE,
        bottom: BUTTON_DARK_EDGE,
    }
}

pub fn pressed_border() -> BorderColor {
    BorderColor {
        top: BUTTON_DARK_EDGE,
        left: BUTTON_DARK_EDGE,
        right: BUTTON_LIGHT_EDGE,
        bottom: BUTTON_LIGHT_EDGE,
    }
}

pub fn hover_border() -> BorderColor {
    BorderColor {
        top: Color::srgb(1.0, 1.0, 1.0),
        left: Color::srgb(0.92, 0.92, 0.92),
        right: Color::srgb(0.08, 0.08, 0.08),
        bottom: Color::srgb(0.04, 0.04, 0.04),
    }
}

pub fn inset_border() -> BorderColor {
    BorderColor {
        top: BUTTON_DARK_EDGE,
        left: BUTTON_DARK_EDGE,
        right: BUTTON_LIGHT_EDGE,
        bottom: BUTTON_LIGHT_EDGE,
    }
}

pub fn update_button_interactions(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<HoverButton>),
    >,
) {
    for (interaction, mut background, mut border) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                *background = BUTTON_PRESSED_BG.into();
                *border = pressed_border();
            }
            Interaction::Hovered => {
                *background = BUTTON_HOVER_BG.into();
                *border = hover_border();
            }
            Interaction::None => {
                *background = BUTTON_BG.into();
                *border = raised_border();
            }
        }
    }
}
