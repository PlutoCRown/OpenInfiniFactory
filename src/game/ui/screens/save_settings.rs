use bevy::prelude::*;

use crate::game::blocks::{BlockKind, all_blocks};
use crate::game::state::UiPanelId;
use crate::game::ui::components::{
    BUTTON_BG, PanelOptions, default_button_size, raised_border, scroll_container, scroll_content,
    spawn_panel, text, text_button,
};
use crate::game::ui::features::save_settings::types::SaveSettingsAction;
use crate::game::ui::types::UiPanelBinding;
use crate::shared::save::{FactoryBlockFilterMode, SaveSettingsData};

/// 存档设置页挂载时使用的只读快照。
#[derive(Clone)]
pub struct SaveSettingsSpawnCtx {
    pub data: SaveSettingsData,
    pub edit_mode: bool,
    pub panel_w: f32,
    pub panel_h: f32,
}

#[derive(Component)]
pub struct SaveSettingsSkyboxPreview;

#[derive(Component, Clone, Copy)]
pub enum SaveSettingsValueText {
    LightPosition,
    LightDirection,
    LightIntensity,
    SolutionSpawn,
    FilterMode,
}

#[derive(Component, Clone, Copy)]
pub struct SaveSettingsBlockIcon(pub BlockKind);

#[derive(Component, Clone, Copy)]
pub struct SaveSettingsFilterMark(pub BlockKind);

#[derive(Component)]
pub struct SaveSettingsCheckMark;

#[derive(Component)]
pub struct SaveSettingsCrossMark;

#[derive(Component)]
pub struct SaveSettingsFactoryPicker;

pub fn save_settings_panel_size(window_w: f32, window_h: f32, ui_scale: f32) -> (f32, f32) {
    let scale = ui_scale.max(0.01);
    (
        (window_w / scale - 72.0).max(1.0),
        (window_h / scale - 72.0).max(1.0),
    )
}

pub fn spawn_save_settings_panel(root: &mut ChildSpawnerCommands, view: &SaveSettingsSpawnCtx) {
    spawn_panel(
        root,
        PanelOptions::new(view.panel_w, "save_settings.title")
            .with_height(view.panel_h)
            .closable(),
        UiPanelBinding(UiPanelId::Settings),
        |panel| {
            panel.spawn(scroll_container()).with_children(|container| {
                container
                    .spawn(scroll_content(0.0))
                    .with_children(|content| {
                        spawn_skybox_section(content);
                        spawn_light_section(content, &view.data);
                        if view.edit_mode {
                            spawn_solution_spawn_section(content, &view.data);
                            spawn_factory_filter_section(content, &view.data);
                        }
                    });
            });
        },
    );
}

fn section_title(parent: &mut ChildSpawnerCommands, title: &'static str) {
    parent.spawn((
        text(
            crate::game::ui::access::i18n.t(title),
            16.0,
            Color::srgb(1.0, 0.90, 0.68),
        ),
        Node {
            margin: UiRect::top(Val::Px(12.0)),
            ..default()
        },
    ));
}

fn action_button(parent: &mut ChildSpawnerCommands, action: SaveSettingsAction, label: String) {
    parent
        .spawn((
            text_button(
                Node {
                    width: Val::Auto,
                    min_width: Val::Px(200.0),
                    height: Val::Px(default_button_size(34.0)),
                    max_width: Val::Px(360.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                raised_border(),
                BUTTON_BG,
            ),
            action,
        ))
        .with_children(|button| {
            button.spawn(text(label, 13.0, Color::WHITE));
        });
}

fn spawn_skybox_section(parent: &mut ChildSpawnerCommands) {
    section_title(parent, "save_settings.skybox");
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(180.0),
                max_width: Val::Px(620.0),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
        ))
        .with_children(|preview| {
            preview.spawn((
                ImageNode::default(),
                SaveSettingsSkyboxPreview,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
    action_button(
        parent,
        SaveSettingsAction::UploadSkybox,
        crate::game::ui::access::i18n.t("save_settings.upload_skybox"),
    );
}

fn value_row(
    parent: &mut ChildSpawnerCommands,
    label: &'static str,
    value: SaveSettingsValueText,
    edit_action: SaveSettingsAction,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(default_button_size(38.0)),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                text(
                    crate::game::ui::access::i18n.t(label),
                    14.0,
                    Color::srgb(0.82, 0.88, 0.90),
                ),
                Node {
                    width: Val::Px(220.0),
                    ..default()
                },
            ));
            row.spawn((
                text_button(
                    Node {
                        width: Val::Auto,
                        max_width: Val::Px(360.0),
                        flex_grow: 1.0,
                        height: Val::Px(default_button_size(34.0)),
                        ..default()
                    },
                    raised_border(),
                    BUTTON_BG,
                ),
                edit_action,
            ))
            .with_children(|button| {
                button.spawn((text(String::new(), 13.0, Color::WHITE), value));
            });
        });
}

fn spawn_light_section(parent: &mut ChildSpawnerCommands, _data: &SaveSettingsData) {
    section_title(parent, "save_settings.global_light");
    value_row(
        parent,
        "save_settings.light_position",
        SaveSettingsValueText::LightPosition,
        SaveSettingsAction::EditLightPosition,
    );
    value_row(
        parent,
        "save_settings.light_direction",
        SaveSettingsValueText::LightDirection,
        SaveSettingsAction::EditLightDirection,
    );
    value_row(
        parent,
        "save_settings.light_intensity",
        SaveSettingsValueText::LightIntensity,
        SaveSettingsAction::EditLightIntensity,
    );
    action_button(
        parent,
        SaveSettingsAction::CaptureLightPose,
        crate::game::ui::access::i18n.t("save_settings.capture_light"),
    );
}

fn spawn_solution_spawn_section(parent: &mut ChildSpawnerCommands, _data: &SaveSettingsData) {
    section_title(parent, "save_settings.solution_spawn");
    value_row(
        parent,
        "save_settings.solution_spawn",
        SaveSettingsValueText::SolutionSpawn,
        SaveSettingsAction::EditSolutionSpawn,
    );
    action_button(
        parent,
        SaveSettingsAction::CaptureSolutionSpawn,
        crate::game::ui::access::i18n.t("save_settings.capture_spawn"),
    );
}

fn spawn_factory_filter_section(parent: &mut ChildSpawnerCommands, data: &SaveSettingsData) {
    section_title(parent, "save_settings.factory_filter");
    value_row(
        parent,
        "save_settings.filter_mode",
        SaveSettingsValueText::FilterMode,
        SaveSettingsAction::OpenFactoryPicker,
    );
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            action_button(
                row,
                SaveSettingsAction::SetFactoryFilterMode(FactoryBlockFilterMode::Blacklist),
                crate::game::ui::access::i18n.t("save_settings.blacklist"),
            );
            action_button(
                row,
                SaveSettingsAction::SetFactoryFilterMode(FactoryBlockFilterMode::Whitelist),
                crate::game::ui::access::i18n.t("save_settings.whitelist"),
            );
        });
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(720.0),
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            SaveSettingsFactoryPicker,
            BackgroundColor(Color::srgb(0.10, 0.11, 0.12)),
        ))
        .with_children(|grid| {
            for kind in all_blocks().into_iter().filter(|kind| kind.is_factory()) {
                spawn_factory_block_button(grid, kind, data);
            }
            action_button(
                grid,
                SaveSettingsAction::CloseFactoryPicker,
                crate::game::ui::access::i18n.t("button.confirm"),
            );
        });
}

fn spawn_factory_block_button(
    parent: &mut ChildSpawnerCommands,
    kind: BlockKind,
    _data: &SaveSettingsData,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(default_button_size(54.0)),
                height: Val::Px(default_button_size(54.0)),
                border: UiRect::all(Val::Px(3.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgb(0.255, 0.251, 0.251)),
            SaveSettingsAction::ToggleFactoryBlock(kind),
        ))
        .with_children(|slot| {
            slot.spawn((
                ImageNode::default(),
                SaveSettingsBlockIcon(kind),
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(4.0),
                    right: Val::Px(4.0),
                    top: Val::Px(4.0),
                    bottom: Val::Px(4.0),
                    ..default()
                },
            ));
            slot.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                SaveSettingsFilterMark(kind),
                Visibility::Hidden,
                Pickable::IGNORE,
            ))
            .with_children(|mark| {
                mark.spawn((Node::default(), SaveSettingsCheckMark, Visibility::Hidden))
                    .with_children(|check| {
                        spawn_mark_line(
                            check,
                            22.0,
                            12.0,
                            25.0,
                            0.75,
                            Color::srgb(0.25, 1.0, 0.35),
                        );
                        spawn_mark_line(
                            check,
                            14.0,
                            7.0,
                            31.0,
                            -0.75,
                            Color::srgb(0.25, 1.0, 0.35),
                        );
                    });
                mark.spawn((Node::default(), SaveSettingsCrossMark, Visibility::Hidden))
                    .with_children(|cross| {
                        spawn_mark_line(cross, 22.0, 8.0, 26.0, 0.75, Color::srgb(1.0, 0.25, 0.25));
                        spawn_mark_line(
                            cross,
                            22.0,
                            8.0,
                            26.0,
                            -0.75,
                            Color::srgb(1.0, 0.25, 0.25),
                        );
                    });
            });
        });
}

fn spawn_mark_line(
    parent: &mut ChildSpawnerCommands,
    width: f32,
    left: f32,
    top: f32,
    rotation: f32,
    color: Color,
) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(width),
            height: Val::Px(4.0),
            left: Val::Px(left),
            top: Val::Px(top),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(color),
        UiTransform::from_rotation(Rot2::radians(rotation)),
    ));
}
