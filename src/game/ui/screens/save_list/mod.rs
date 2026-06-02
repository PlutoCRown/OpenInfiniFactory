use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::state::GameMode;
use crate::shared::i18n::I18n;

use crate::game::ui::components::{
    default_button_size, default_font_size, flex_row, panel_close_label_scene, panel_content_scene,
    panel_title_bar_scene, panel_title_button_scene, panel_title_label_scene, panel_window_scene,
    raised_border, text, HoverButton, BUTTON_BG,
};
use crate::game::ui::types::{
    PanelPosition, PanelText, PanelTextKind, PanelTitleBar, PanelVisibility, PanelWindow,
    SaveListAction, SaveListCloseButton, SaveListPanel, SaveListPrompt, SaveListPuzzleColumn,
    SaveListSolutionColumn, TextPromptAction, TextPromptRoot, TextPromptText,
};

mod actions;
mod systems;

pub(crate) use actions::{save_list_actions, text_prompt_actions, text_prompt_input};
pub use systems::{update_save_list_ui, update_text_prompt_ui};

pub fn spawn_save_list(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    let panel = root
        .spawn((
            GlobalZIndex(0),
            PanelWindow,
            PanelPosition::default(),
            Visibility::Hidden,
            PanelVisibility::GameMode(GameMode::SaveListMain),
            SaveListPanel,
        ))
        .queue_apply_scene(panel_window_scene(900.0))
        .with_children(|panel| {
            panel
                .spawn((Button, PanelTitleBar))
                .queue_apply_scene(panel_title_bar_scene())
                .with_children(|title| {
                    title
                        .spawn(PanelText(PanelTextKind::SaveListTitle))
                        .queue_apply_scene(panel_title_label_scene(
                            i18n.text("save.title.default"),
                            26.0,
                        ));
                    title
                        .spawn((
                            Button,
                            HoverButton,
                            SaveListAction::Back,
                            SaveListCloseButton,
                        ))
                        .queue_apply_scene(panel_title_button_scene())
                        .queue_spawn_related_scenes::<Children>(panel_close_label_scene());
                });
            panel
                .spawn_empty()
                .queue_apply_scene(panel_content_scene())
                .with_children(|panel| {
                    panel
                        .spawn_empty()
                        .queue_apply_scene(save_columns_row_scene())
                        .with_children(|columns| {
                            spawn_save_column(
                                columns,
                                SaveListAction::NewPuzzle,
                                SaveListPuzzleColumn,
                            );
                            spawn_save_column(
                                columns,
                                SaveListAction::NewSolution,
                                SaveListSolutionColumn,
                            );
                        });
                    panel.spawn((
                        text("", 16.0, Color::srgb(0.82, 0.86, 0.88)),
                        SaveListPrompt,
                    ));
                });
        })
        .id();
    spawn_text_prompt(root);
    panel
}

fn save_columns_row_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Auto,
            display: Display::Flex,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(12.0),
        }
        BackgroundColor(Color::NONE)
    }
}

fn spawn_save_column(
    columns: &mut ChildSpawnerCommands,
    create: SaveListAction,
    marker: impl Component + Copy,
) {
    columns
        .spawn(marker)
        .queue_apply_scene(save_column_scene())
        .with_children(|column| {
            spawn_save_slot_button(column, create);
        });
}

fn save_column_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(SAVE_LIST_EDIT_COLUMN_WIDTH),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
        }
        BackgroundColor(Color::NONE)
    }
}

pub(super) fn spawn_save_slot_button(parent: &mut ChildSpawnerCommands, action: SaveListAction) {
    parent
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(save_full_width_button_scene(34.0))
        .queue_spawn_related_scenes::<Children>(save_button_label_scene("", 15.0));
}

pub fn spawn_save_management_row(
    parent: &mut ChildSpawnerCommands,
    load: SaveListAction,
    rename: SaveListAction,
    delete: SaveListAction,
    width: f32,
) {
    let side_width = 82.0;
    let load_width = (width - side_width * 2.0 - 12.0).max(180.0);
    parent.spawn(flex_row(32.0, 6.0)).with_children(|row| {
        spawn_save_row_button(row, load, load_width);
        spawn_save_row_button(row, rename, side_width);
        spawn_save_row_button(row, delete, side_width);
    });
}

pub fn spawn_save_select_row(parent: &mut ChildSpawnerCommands, load: SaveListAction) {
    parent
        .spawn((Button, HoverButton, load))
        .queue_apply_scene(save_full_width_button_scene(32.0))
        .queue_spawn_related_scenes::<Children>(save_button_label_scene("", 13.0));
}

fn spawn_save_row_button(parent: &mut ChildSpawnerCommands, action: SaveListAction, width: f32) {
    parent
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(save_fixed_width_button_scene(width, 30.0))
        .queue_spawn_related_scenes::<Children>(save_button_label_scene("", 13.0));
}

fn spawn_text_prompt(root: &mut ChildSpawnerCommands) {
    root.spawn((
        GlobalZIndex(30_000),
        PanelWindow,
        PanelPosition::default(),
        Visibility::Hidden,
        TextPromptRoot,
    ))
    .queue_apply_scene(panel_window_scene(420.0))
    .with_children(|panel| {
        panel
            .spawn((Button, PanelTitleBar))
            .queue_apply_scene(panel_title_bar_scene())
            .with_children(|title| {
                title
                    .spawn(TextPromptText::Title)
                    .queue_apply_scene(panel_title_label_scene(String::new(), 20.0));
            });
        panel
            .spawn_empty()
            .queue_apply_scene(panel_content_scene())
            .with_children(|content| {
                content
                    .spawn((Button, HoverButton))
                    .queue_apply_scene(text_prompt_input_scene())
                    .with_children(|input| {
                        input
                            .spawn(TextPromptText::Value)
                            .queue_apply_scene(text_prompt_value_scene());
                    });
                content.spawn(flex_row(36.0, 8.0)).with_children(|row| {
                    spawn_prompt_button(row, TextPromptAction::Confirm);
                    spawn_prompt_button(row, TextPromptAction::Cancel);
                });
            });
    });
}

fn spawn_prompt_button(parent: &mut ChildSpawnerCommands, action: TextPromptAction) {
    parent
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(save_full_width_button_scene(34.0))
        .queue_spawn_related_scenes::<Children>(save_button_label_scene("", 15.0));
}

fn save_full_width_button_scene(height: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            height: Val::Px(default_button_size(height)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            padding: UiRect::horizontal(Val::Px(14.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        )
    }
}

fn save_fixed_width_button_scene(width: f32, height: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(default_button_size(width)),
            min_width: Val::Px(default_button_size(width)),
            height: Val::Px(default_button_size(height)),
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
    }
}

fn save_button_label_scene(value: &'static str, font_size: f32) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({value})
            TextFont {
                font_size: {default_font_size(font_size)}
            }
            TextColor(Color::WHITE)
        )
    }
}

fn text_prompt_input_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(default_button_size(38.0)),
            padding: UiRect::horizontal(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
    }
}

fn text_prompt_value_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(16.0)}
        }
        TextColor(Color::WHITE)
    }
}

const SAVE_LIST_EDIT_COLUMN_WIDTH: f32 = 466.0;
