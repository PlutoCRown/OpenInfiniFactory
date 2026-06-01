use bevy::prelude::*;

use crate::shared::i18n::I18n;

use super::components::{absolute_text_bundle, root_node, STATUS_TEXT};
use super::screens::{
    spawn_carried_label, spawn_confirm_dialog, spawn_hotbar, spawn_inventory_panel,
    spawn_inventory_tooltip, spawn_main_menu, spawn_pause_panel, spawn_save_list,
    spawn_settings_panel,
};
use super::types::{
    Crosshair, GameplayHudVisibility, InGameHudVisibility, PanelVisibility, StatusText,
    StatusTextKind,
};
use crate::game::world::blocks::{spawn_block_dropdown_layers, spawn_block_panels};

pub fn setup_ui(mut commands: Commands, i18n: Res<I18n>) {
    commands.spawn(root_node()).with_children(|root| {
        spawn_status_overlays(root);
        spawn_hotbar(root);
        spawn_inventory_panel(root, &i18n);
        spawn_block_panels(root, &i18n);
        spawn_pause_panel(root, &i18n);
        spawn_settings_panel(root, &i18n);
        spawn_confirm_dialog(root);
        spawn_modal_scrim(root);
        spawn_main_menu(root, &i18n);
        spawn_save_list(root, &i18n);
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
    root.spawn((
        absolute_text_bundle(
            "+",
            30.0,
            Color::WHITE,
            Some(Val::Percent(50.0)),
            None,
            Some(Val::Percent(50.0)),
            None,
        ),
        Crosshair,
        InGameHudVisibility,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            Color::WHITE,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(62.0)),
            None,
        ),
        StatusText(StatusTextKind::Hotbar),
        InGameHudVisibility,
        GameplayHudVisibility,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            15.0,
            STATUS_TEXT,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(18.0)),
            None,
        ),
        StatusText(StatusTextKind::CurrentSave),
        InGameHudVisibility,
        GameplayHudVisibility,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            STATUS_TEXT,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(112.0)),
            None,
        ),
        StatusText(StatusTextKind::Simulation),
        InGameHudVisibility,
        GameplayHudVisibility,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            STATUS_TEXT,
            None,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(18.0)),
        ),
        StatusText(StatusTextKind::SimulationOverlay),
        InGameHudVisibility,
    ));
}
