use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Out, Over, Pointer, Press, Release};
use bevy::prelude::*;

use super::text::default_button_size;

pub const BUTTON_BG: Color = Color::srgb(0.56, 0.56, 0.56);
pub const BUTTON_HOVER_BG: Color = Color::srgb(0.68, 0.68, 0.68);
pub const BUTTON_PRESSED_BG: Color = Color::srgb(0.40, 0.40, 0.40);
/// 凸起亮边（左/上）
pub const BUTTON_LIGHT_EDGE: Color = Color::srgb(0.89, 0.89, 0.89);
/// 凸起暗边（右/下），刻意不要纯黑
pub const BUTTON_DARK_EDGE: Color = Color::srgb(0.30, 0.30, 0.30);
/// 悬停时更亮/更暗的边
pub const BUTTON_HOVER_LIGHT_EDGE: Color = Color::srgb(1.0, 1.0, 1.0);
pub const BUTTON_HOVER_LIGHT_EDGE_SOFT: Color = Color::srgb(0.94, 0.94, 0.94);
pub const BUTTON_HOVER_DARK_EDGE: Color = Color::srgb(0.22, 0.22, 0.22);
pub const BUTTON_HOVER_DARK_EDGE_SOFT: Color = Color::srgb(0.26, 0.26, 0.26);

/// 按钮边框厚度（略不对称保留立体感）
pub const BUTTON_BORDER_X: f32 = 2.0;
pub const BUTTON_BORDER_TOP: f32 = 2.0;
pub const BUTTON_BORDER_BOTTOM: f32 = 2.5;
pub const BUTTON_PAD_X: f32 = 14.0;

/// 按钮外阴影（右下角落下感）
pub const BUTTON_SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.28);
pub const BUTTON_SHADOW_BLUR: f32 = 3.0;

#[derive(Component)]
pub struct HoverButton;

/// 凸起按钮的边框宽度
pub fn button_border() -> UiRect {
    UiRect {
        left: Val::Px(BUTTON_BORDER_X),
        right: Val::Px(BUTTON_BORDER_X),
        top: Val::Px(BUTTON_BORDER_TOP),
        bottom: Val::Px(BUTTON_BORDER_BOTTOM),
    }
}

/// 凸起按钮的外阴影
pub fn button_shadow() -> BoxShadow {
    BoxShadow::new(
        BUTTON_SHADOW,
        Val::Px(0.0),
        Val::Px(0.0),
        Val::Px(0.0),
        Val::Px(BUTTON_SHADOW_BLUR),
    )
}

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

pub fn menu_button(height: f32) -> impl Bundle {
    text_button(
        Node {
            height: Val::Px(default_button_size(height)),
            ..default()
        },
        raised_border(),
        BUTTON_BG,
    )
}

pub fn full_width_button(height: f32) -> impl Bundle {
    text_button(
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            height: Val::Px(default_button_size(height)),
            ..default()
        },
        raised_border(),
        BUTTON_BG,
    )
}

pub fn auto_width_button(height: f32) -> impl Bundle {
    text_button(
        Node {
            width: Val::Auto,
            height: Val::Px(default_button_size(height)),
            flex_shrink: 0.0,
            ..default()
        },
        raised_border(),
        BUTTON_BG,
    )
}

pub fn text_button(style: Node, border: impl Into<BorderColor>, background: Color) -> impl Bundle {
    let border = border.into();
    (
        styled_button(
            Node {
                border: button_border(),
                padding: UiRect::horizontal(Val::Px(BUTTON_PAD_X)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..style
            },
            border,
            background,
        ),
        button_shadow(),
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
        top: BUTTON_HOVER_LIGHT_EDGE,
        left: BUTTON_HOVER_LIGHT_EDGE_SOFT,
        right: BUTTON_HOVER_DARK_EDGE,
        bottom: BUTTON_HOVER_DARK_EDGE_SOFT,
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

pub fn button_hovered(
    mut event: On<Pointer<Over>>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<HoverButton>>,
) {
    let Ok((mut background, mut border)) = buttons.get_mut(event.entity) else {
        return;
    };
    event.propagate(false);
    *background = BUTTON_HOVER_BG.into();
    *border = hover_border();
}

pub fn button_unhovered(
    mut event: On<Pointer<Out>>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<HoverButton>>,
) {
    let Ok((mut background, mut border)) = buttons.get_mut(event.entity) else {
        return;
    };
    event.propagate(false);
    *background = BUTTON_BG.into();
    *border = raised_border();
}

pub fn button_pressed(
    mut event: On<Pointer<Press>>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<HoverButton>>,
) {
    if event.event.button != PointerButton::Primary {
        return;
    }
    let Ok((mut background, mut border)) = buttons.get_mut(event.entity) else {
        return;
    };
    event.propagate(false);
    *background = BUTTON_PRESSED_BG.into();
    *border = pressed_border();
}

pub fn button_released(
    mut event: On<Pointer<Release>>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<HoverButton>>,
) {
    if event.event.button != PointerButton::Primary {
        return;
    }
    let Ok((mut background, mut border)) = buttons.get_mut(event.entity) else {
        return;
    };
    event.propagate(false);
    *background = BUTTON_HOVER_BG.into();
    *border = hover_border();
}
