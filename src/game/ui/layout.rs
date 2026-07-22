use bevy::prelude::*;

use crate::game::ui::access::{bind_ui_scope, unbind_ui_scope};

use super::components::{STATUS_TEXT, absolute_text_bundle, root_node};
use super::screens::{
    spawn_carried_label, spawn_hotbar, spawn_inventory_panel, spawn_inventory_tooltip,
};
use super::types::{
    Crosshair, InGameHudVisibility, PlayingUiRoot, StatusText, StatusTextKind, UiRoot,
};
use crate::game::cameras::{GameplayViewBackdrop, GameplayViewImage};
use crate::game::session::SessionBusy;
use crate::game::state::BuilderMode;
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHostMountRoot, UiRootEntity};
use crate::game::ui::features::playing_overlays::PlayingOverlayMounts;
use crate::game::ui::features::session_busy::spawn_session_busy_overlay;
use crate::game::ui::features::virtual_remote::spawn_virtual_remote;
use crate::shared::touch_profile::TouchProfile;

/// 启动时只建空菜单根；主菜单/存档列表/对话框按需挂载
pub fn setup_menu_ui(world: &mut World) {
    bind_ui_scope(world);
    let mut commands = world.commands();
    let root = commands.spawn((root_node(), UiRoot)).id();
    commands.insert_resource(UiRootEntity(root));
    unbind_ui_scope(world);
}

fn spawn_gameplay_view_backdrop(root: &mut ChildSpawnerCommands, image: Handle<Image>) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode::new(image),
        GameplayViewBackdrop,
        Pickable::IGNORE,
    ));
}

pub fn setup_playing_ui_system(world: &mut World) {
    bind_ui_scope(world);
    let Some(view) = world.get_resource::<GameplayViewImage>() else {
        unbind_ui_scope(world);
        return;
    };
    let image = view.0.clone();
    let touch = world
        .get_resource::<TouchProfile>()
        .copied()
        .unwrap_or(TouchProfile { enabled: false });
    let builder_mode = world
        .get_resource::<BuilderMode>()
        .copied()
        .unwrap_or(BuilderMode::Edit);
    let busy = world
        .get_resource::<SessionBusy>()
        .copied()
        .unwrap_or_default();
    let mut commands = world.commands();
    let inventory = setup_playing_ui(&mut commands, image, touch, builder_mode, busy);
    commands.insert_resource(PlayingOverlayMounts {
        inventory: Some(inventory),
        pause: None,
    });
    unbind_ui_scope(world);
}

/// 返回背包挂载根实体（常驻，用 Display 显隐）
pub fn setup_playing_ui(
    commands: &mut Commands,
    view_image: Handle<Image>,
    touch: TouchProfile,
    builder_mode: BuilderMode,
    busy: SessionBusy,
) -> Entity {
    let mut inventory_mount = None;
    let root = commands
        .spawn((root_node(), PlayingUiRoot))
        .with_children(|root| {
            spawn_gameplay_view_backdrop(root, view_image);
            spawn_status_overlays(root);
            spawn_hotbar(root);
            spawn_carried_label(root);
            spawn_inventory_tooltip(root);
            spawn_session_busy_overlay(root, busy);
            spawn_virtual_remote(root, &touch, false);
            inventory_mount = Some(
                root.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    UiHostMountRoot,
                    Pickable::IGNORE,
                ))
                .with_children(|container| {
                    spawn_inventory_panel(container, builder_mode, touch);
                })
                .id(),
            );
        })
        .id();
    commands.insert_resource(PlayingUiRootEntity(root));
    inventory_mount.expect("inventory mount")
}

const CROSSHAIR_ARM: f32 = 12.0;
const CROSSHAIR_THICKNESS: f32 = 2.0;

fn spawn_crosshair(root: &mut ChildSpawnerCommands) {
    let offset = (CROSSHAIR_ARM - CROSSHAIR_THICKNESS) * 0.5;

    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Crosshair,
        InGameHudVisibility,
        // 高于 3D 底图，低于热栏(5) / 面板，避免被后挂载的全屏透明层盖住
        GlobalZIndex(1),
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
    ))
    .with_children(|overlay| {
        // 先放一个被 Flex 居中的锚点，再在其内画十字（绝对定位相对锚点）
        overlay
            .spawn(Node {
                width: Val::Px(CROSSHAIR_ARM),
                height: Val::Px(CROSSHAIR_ARM),
                ..default()
            })
            .with_children(|mark| {
                mark.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(offset),
                        width: Val::Px(CROSSHAIR_ARM),
                        height: Val::Px(CROSSHAIR_THICKNESS),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    Pickable::IGNORE,
                ));
                mark.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(offset),
                        top: Val::Px(0.0),
                        width: Val::Px(CROSSHAIR_THICKNESS),
                        height: Val::Px(CROSSHAIR_ARM),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    Pickable::IGNORE,
                ));
            });
    });
}

fn spawn_status_overlays(root: &mut ChildSpawnerCommands) {
    spawn_crosshair(root);
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            STATUS_TEXT,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(18.0)),
            None,
        ),
        TextLayout::no_wrap(),
        StatusText(StatusTextKind::Gameplay),
        InGameHudVisibility,
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
        TextLayout::no_wrap(),
        StatusText(StatusTextKind::SimulationOverlay),
        InGameHudVisibility,
    ));
}
