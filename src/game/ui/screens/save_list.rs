use bevy::prelude::*;

use crate::game::state::StartMenuScreen;
use crate::game::ui::components::{
    BUTTON_BG, PanelOptions, UiIconAssets, button_border, button_shadow, default_button_size,
    raised_border, spawn_panel_with_title, spawn_ui_icon, styled_button, text, text_button,
    transparent_node,
};

use super::super::types::{
    LocalizedText, PanelVisibility, SaveListAction, SaveListCloseButton, SaveListCoverHost,
    SaveListCoverImage, SaveListCoverLoading, SaveListPanel, SaveListPuzzleRows,
    SaveListPuzzleScroll, SaveListSolutionRows, SaveListSolutionScroll, SaveListTitleText,
};

/// 相对窗口逻辑像素的外边距
const SAVE_LIST_MARGIN: f32 = 48.0;
const PUZZLE_COL_WIDTH_PERCENT: f32 = 25.0;
const SOLUTION_CARD_WIDTH: f32 = 140.0;
/// 方案横滑区内边距（与卡片高度一起决定条带高度）
const SOLUTION_STRIP_PAD: f32 = 6.0;

/// 存档弹窗挂载时预解析的文案与图标（须在 UiAccessScope 内算好再传入）
pub struct SaveListSpawnCtx {
    pub title: String,
    pub puzzle_heading: String,
    pub solution_heading: String,
    pub icons: UiIconAssets,
}

/// 按窗口逻辑尺寸算出弹窗宽高（仅初始化用）
pub fn save_list_panel_size(window_w: f32, window_h: f32, ui_scale: f32) -> (f32, f32) {
    let scale = ui_scale.max(0.01);
    (
        (window_w / scale - SAVE_LIST_MARGIN * 2.0).max(480.0),
        (window_h / scale - SAVE_LIST_MARGIN * 2.0).max(320.0),
    )
}

/// 挂载统一存档选择弹窗（文案/图标须已在外部解析好）
pub fn spawn_save_list(
    root: &mut ChildSpawnerCommands,
    panel_w: f32,
    panel_h: f32,
    ctx: SaveListSpawnCtx,
) {
    let SaveListSpawnCtx {
        title,
        puzzle_heading,
        solution_heading,
        icons,
    } = ctx;

    spawn_panel_with_title(
        root,
        PanelOptions::new(panel_w, "save.title.play_solution")
            .with_height(panel_h)
            .closable(),
        (
            SaveListPanel,
            PanelVisibility::StartMenuScreen(StartMenuScreen::SaveList),
        ),
        title,
        SaveListTitleText,
        (SaveListAction::Back, SaveListCloseButton),
        |content| {
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|body| {
                    spawn_puzzle_column(body, puzzle_heading);
                    spawn_right_column(body, solution_heading);
                });
            spawn_footer(content, &icons);
        },
    );
}

fn spawn_puzzle_column(body: &mut ChildSpawnerCommands, heading: String) {
    body.spawn((
        Node {
            width: Val::Percent(PUZZLE_COL_WIDTH_PERCENT),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            min_width: Val::Px(0.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|col| {
        col.spawn((
            text(heading, 14.0, Color::srgb(0.85, 0.88, 0.9)),
            LocalizedText {
                key: "save.title.select_puzzle_list",
            },
        ));
        col.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::NONE),
            SaveListPuzzleScroll {
                offset: 0.0,
                max_offset: 0.0,
            },
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .with_children(|viewport| {
            viewport.spawn((
                transparent_node(Node {
                    width: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                }),
                SaveListPuzzleRows,
            ));
        });
    });
}

fn spawn_right_column(body: &mut ChildSpawnerCommands, heading: String) {
    body.spawn(Node {
        width: Val::Percent(100.0 - PUZZLE_COL_WIDTH_PERCENT),
        height: Val::Percent(100.0),
        flex_grow: 1.0,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(8.0),
        min_width: Val::Px(0.0),
        overflow: Overflow::clip(),
        ..default()
    })
    .with_children(|col| {
        col.spawn((
            text(heading, 14.0, Color::srgb(0.85, 0.88, 0.9)),
            LocalizedText {
                key: "save.title.select_solution",
            },
        ));

        col.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(solution_card_height() + SOLUTION_STRIP_PAD * 2.0),
                overflow: Overflow::clip(),
                position_type: PositionType::Relative,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.18)),
            SaveListSolutionScroll {
                offset: 0.0,
                max_offset: 0.0,
            },
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .with_children(|viewport| {
            viewport.spawn((
                Node {
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(SOLUTION_STRIP_PAD)),
                    align_items: AlignItems::Stretch,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                SaveListSolutionRows,
            ));
        });

        col.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                overflow: Overflow::clip(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.22)),
            SaveListCoverHost,
        ))
        .with_children(|host| {
            host.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::None,
                    ..default()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                SaveListCoverImage,
                Pickable::IGNORE,
            ));
            host.spawn((
                text("…", 16.0, Color::srgb(0.72, 0.76, 0.8)),
                LocalizedText {
                    key: "save.cover_loading",
                },
                SaveListCoverLoading,
                Visibility::Hidden,
                Pickable::IGNORE,
            ));
        });
    });
}

fn spawn_footer(panel: &mut ChildSpawnerCommands, icons: &UiIconAssets) {
    let btn_h = default_button_size(34.0);
    let btn_node = Node {
        width: Val::Auto,
        height: Val::Px(btn_h),
        ..default()
    };
    // 仅图标：与关闭按钮同为方形（宽=高），高度跟齐文字按钮
    let icon_btn_node = Node {
        width: Val::Px(btn_h),
        height: Val::Px(btn_h),
        border: button_border(),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_shrink: 0.0,
        ..default()
    };
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            flex_shrink: 0.0,
            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
            ..default()
        })
        .with_children(|footer| {
            footer
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        text_button(btn_node.clone(), raised_border(), BUTTON_BG),
                        SaveListAction::NewPuzzle,
                    ))
                    .with_children(|button| {
                        button.spawn(text("", 15.0, Color::WHITE));
                    });
                    left.spawn((
                        text_button(btn_node.clone(), raised_border(), BUTTON_BG),
                        SaveListAction::EditSelectedPuzzle,
                    ))
                    .with_children(|button| {
                        button.spawn(text("", 15.0, Color::WHITE));
                    });
                    left.spawn((
                        styled_button(icon_btn_node.clone(), raised_border(), BUTTON_BG),
                        button_shadow(),
                        SaveListAction::RenameSelectedPuzzle,
                    ))
                    .with_children(|button| {
                        spawn_ui_icon(button, icons.edit.clone(), 16.0);
                    });
                    left.spawn((
                        styled_button(icon_btn_node.clone(), raised_border(), BUTTON_BG),
                        button_shadow(),
                        SaveListAction::DeleteSelectedPuzzle,
                    ))
                    .with_children(|button| {
                        spawn_ui_icon(button, icons.delete.clone(), 16.0);
                    });
                });

            footer
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|right| {
                    right
                        .spawn((
                            styled_button(icon_btn_node.clone(), raised_border(), BUTTON_BG),
                            button_shadow(),
                            SaveListAction::DeleteSelectedSolution,
                        ))
                        .with_children(|button| {
                            spawn_ui_icon(button, icons.delete.clone(), 16.0);
                        });
                    right
                        .spawn((
                            styled_button(icon_btn_node, raised_border(), BUTTON_BG),
                            button_shadow(),
                            SaveListAction::RenameSelectedSolution,
                        ))
                        .with_children(|button| {
                            spawn_ui_icon(button, icons.edit.clone(), 16.0);
                        });
                    right
                        .spawn((
                            text_button(btn_node, raised_border(), BUTTON_BG),
                            SaveListAction::StartGame,
                        ))
                        .with_children(|button| {
                            button.spawn(text("", 15.0, Color::WHITE));
                        });
                });
        });
}

/// 挂载一条谜题选择行
pub fn spawn_save_puzzle_row(parent: &mut ChildSpawnerCommands, storage: String) {
    parent
        .spawn((
            text_button(
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(default_button_size(32.0)),
                    flex_shrink: 0.0,
                    ..default()
                },
                raised_border(),
                BUTTON_BG,
            ),
            SaveListAction::SelectPuzzle(storage),
        ))
        .with_children(|button| {
            button.spawn(save_row_label("", 13.0));
        });
}

/// 挂载一张方案卡片
pub fn spawn_save_solution_card(parent: &mut ChildSpawnerCommands, storage: Option<String>) {
    let action = match storage {
        Some(name) => SaveListAction::SelectSolution(name),
        None => SaveListAction::NewSolution,
    };
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(SOLUTION_CARD_WIDTH),
                    height: Val::Px(solution_card_height()),
                    border: button_border(),
                    padding: UiRect::all(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_shrink: 0.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                raised_border(),
                BUTTON_BG,
            ),
            button_shadow(),
            action,
        ))
        .with_children(|card| {
            card.spawn(save_row_label("", 13.0));
        });
}

/// 方案卡高度：与左侧 4 个谜题行按钮等高
fn solution_card_height() -> f32 {
    4.0 * default_button_size(32.0)
}

fn save_row_label(value: impl Into<String>, font_size: f32) -> impl Bundle {
    (
        text(value, font_size, Color::WHITE),
        TextLayout::no_wrap(),
        Node {
            max_width: Val::Percent(100.0),
            overflow: Overflow::clip(),
            ..default()
        },
    )
}
