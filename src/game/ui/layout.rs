use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::shared::i18n::I18n;

use super::components::{default_font_size, root_node, STATUS_TEXT};
use super::screens::{spawn_carried_label, spawn_hotbar, spawn_inventory_tooltip};
use super::types::{
    Crosshair, GameplayHudVisibility, InGameHudVisibility, PanelVisibility, StatusText,
    StatusTextKind, UiRoot,
};
use crate::game::world::blocks::spawn_block_dropdown_layers;

pub fn setup_ui(mut commands: Commands, i18n: Res<I18n>) {
    commands.spawn((root_node(), UiRoot)).with_children(|root| {
        spawn_status_overlays(root);
        spawn_hotbar(root);
        spawn_modal_scrim(root);
        spawn_carried_label(root);
        spawn_inventory_tooltip(root);
        spawn_block_dropdown_layers(root, &i18n);
    });
}

fn spawn_modal_scrim(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.16)),
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
        GlobalZIndex(0),
        PanelVisibility::ModalScrim,
    ));
}

fn spawn_status_overlays(root: &mut ChildSpawnerCommands) {
    root.spawn((Crosshair, InGameHudVisibility))
        .queue_apply_scene(status_text_scene(
            "+",
            30.0,
            Color::WHITE,
            Some(Val::Percent(50.0)),
            None,
            Some(Val::Percent(50.0)),
            None,
        ));
    root.spawn((
        StatusText(StatusTextKind::Hotbar),
        InGameHudVisibility,
        GameplayHudVisibility,
    ))
    .queue_apply_scene(status_text_scene(
        "",
        16.0,
        Color::WHITE,
        Some(Val::Px(18.0)),
        None,
        Some(Val::Px(62.0)),
        None,
    ));
    root.spawn((
        StatusText(StatusTextKind::CurrentSave),
        InGameHudVisibility,
        GameplayHudVisibility,
    ))
    .queue_apply_scene(status_text_scene(
        "",
        15.0,
        STATUS_TEXT,
        Some(Val::Px(18.0)),
        None,
        Some(Val::Px(18.0)),
        None,
    ));
    root.spawn((
        StatusText(StatusTextKind::Simulation),
        InGameHudVisibility,
        GameplayHudVisibility,
    ))
    .queue_apply_scene(status_text_scene(
        "",
        16.0,
        STATUS_TEXT,
        Some(Val::Px(18.0)),
        None,
        Some(Val::Px(112.0)),
        None,
    ));
    root.spawn((
        StatusText(StatusTextKind::SimulationOverlay),
        InGameHudVisibility,
    ))
    .queue_apply_scene(status_text_scene(
        "",
        16.0,
        STATUS_TEXT,
        None,
        Some(Val::Px(18.0)),
        None,
        Some(Val::Px(18.0)),
    ));
}

fn status_text_scene(
    value: &'static str,
    font_size: f32,
    color: Color,
    left: Option<Val>,
    right: Option<Val>,
    top: Option<Val>,
    bottom: Option<Val>,
) -> impl bevy_scene::Scene {
    let left = left.unwrap_or(Val::Auto);
    let right = right.unwrap_or(Val::Auto);
    let top = top.unwrap_or(Val::Auto);
    let bottom = bottom.unwrap_or(Val::Auto);
    bsn! {
        Text({value})
        TextFont {
            font_size: {default_font_size(font_size)}
        }
        TextColor(color)
        Visibility::Hidden
        Node {
            position_type: PositionType::Absolute,
            left: {left},
            right: {right},
            top: {top},
            bottom: {bottom},
        }
        Pickable::IGNORE
    }
}
