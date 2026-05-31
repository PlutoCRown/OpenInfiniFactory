use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, TrackClick};

use super::button::{inset_border, styled_button};
use super::text::default_button_size;

#[derive(Component)]
pub struct SliderFill;

#[derive(Component)]
pub struct SliderKnob;

pub fn slider_bundle(action: impl Component + Copy) -> impl Bundle {
    (
        styled_button(
            Node {
                width: Val::Px(360.0),
                height: Val::Px(default_button_size(22.0)),
                padding: UiRect::all(Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            inset_border(),
            Color::srgb(0.255, 0.251, 0.251),
        ),
        Slider {
            track_click: TrackClick::Snap,
        },
        SliderValue(0.0),
        SliderRange::new(0.0, 100.0),
        action,
    )
}

pub fn slider_fill() -> impl Bundle {
    (
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.32, 0.62, 0.72)),
        SliderFill,
        Pickable::IGNORE,
    )
}

pub fn slider_knob() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Px(3.0),
            bottom: Val::Px(3.0),
            width: Val::Px(6.0),
            margin: UiRect {
                left: Val::Px(-3.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::srgb(0.90, 0.96, 1.0)),
        SliderKnob,
        SliderThumb,
        Pickable::IGNORE,
    )
}
