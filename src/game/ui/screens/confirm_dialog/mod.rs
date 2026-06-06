use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{
    default_button_size, default_font_size, panel_content_scene, panel_title_bar_scene,
    panel_window_scene, raised_border, HoverButton, BUTTON_BG, STATUS_TEXT,
};
use crate::game::ui::types::{
    ConfirmDialogAction, ConfirmDialogMessage, ConfirmDialogRoot, ConfirmDialogState,
    LanguageChanged, LocalizedText, PanelPosition, PanelText, PanelTextKind, PanelTitleBar,
    PanelVisibility, PanelWindow, UiModalClosed, UiModalKind, UiModalOpened, UiRuntime,
};
use crate::shared::i18n::I18n;

mod actions;

pub(crate) use actions::confirm_dialog_actions;

pub fn spawn_confirm_dialog(root: &mut ChildSpawnerCommands) {
    root.spawn((
        GlobalZIndex(0),
        PanelWindow,
        PanelPosition::default(),
        Visibility::Visible,
        PanelVisibility::ConfirmDialog,
        ConfirmDialogRoot,
    ))
    .queue_apply_scene(panel_window_scene(620.0))
    .with_children(|panel| {
        panel
            .spawn((Button, PanelTitleBar))
            .queue_apply_scene(panel_title_bar_scene())
            .with_children(|title| {
                title
                    .spawn(PanelText(PanelTextKind::ConfirmTitle))
                    .queue_apply_scene(confirm_dialog_title_scene());
            });
        panel
            .spawn(Visibility::Visible)
            .queue_apply_scene(panel_content_scene())
            .with_children(|panel| {
                panel
                    .spawn(PanelText(PanelTextKind::ConfirmMessage))
                    .queue_apply_scene(confirm_dialog_message_scene());
                panel
                    .spawn(Visibility::Visible)
                    .queue_apply_scene(confirm_dialog_actions_row_scene())
                    .with_children(|row| {
                        spawn_confirm_dialog_button(
                            row,
                            ConfirmDialogAction::Primary,
                            "button.confirm",
                        );
                        spawn_confirm_dialog_button(
                            row,
                            ConfirmDialogAction::Secondary,
                            "button.confirm",
                        );
                        spawn_confirm_dialog_button(
                            row,
                            ConfirmDialogAction::Cancel,
                            "button.cancel",
                        );
                    });
            });
    });
}

fn confirm_dialog_title_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        Pickable::IGNORE
        TextFont {
            font_size: {default_font_size(24.0 * 0.8)}
        }
        TextColor(Color::srgb(1.0, 0.902, 0.753))
        Node {
            flex_grow: 1.0,
        }
    }
}

fn confirm_dialog_message_scene() -> impl bevy_scene::Scene {
    bsn! {
        Text("")
        TextFont {
            font_size: {default_font_size(15.0)}
        }
        TextColor(STATUS_TEXT)
        TextLayout::justify(Justify::Center)
        Node {
            min_height: Val::Px(54.0),
            align_self: AlignSelf::Stretch,
        }
    }
}

pub fn update_confirm_dialog_ui(
    ui_runtime: Res<UiRuntime>,
    i18n: Res<I18n>,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, (With<LocalizedText>, Without<PanelText>)>,
    )>,
    added_panel_texts: Query<(), Added<PanelText>>,
    added_actions: Query<(), Added<ConfirmDialogAction>>,
    mut modal_opened: MessageReader<UiModalOpened>,
    mut modal_closed: MessageReader<UiModalClosed>,
    mut language_changed: MessageReader<LanguageChanged>,
    mut action_buttons: Query<(&ConfirmDialogAction, &mut Node, &Children), With<Button>>,
) {
    let modal_dirty = modal_opened
        .read()
        .any(|message| message.kind == UiModalKind::ConfirmDialog)
        || modal_closed
            .read()
            .any(|message| message.kind == UiModalKind::ConfirmDialog);

    if !modal_dirty
        && language_changed.read().next().is_none()
        && added_panel_texts.is_empty()
        && added_actions.is_empty()
    {
        return;
    }

    let Some(dialog) = ui_runtime.confirm_dialog() else {
        return;
    };
    for (panel_text, mut text) in &mut texts.p0() {
        text.0 = match panel_text.0 {
            PanelTextKind::ConfirmTitle => i18n.text(dialog.title_key),
            PanelTextKind::ConfirmMessage => confirm_dialog_message(&dialog.message, &i18n),
            _ => continue,
        };
    }
    for (action, mut node, children) in &mut action_buttons {
        if matches!(*action, ConfirmDialogAction::Secondary) {
            node.display = if dialog.secondary_key.is_some() {
                Display::Flex
            } else {
                Display::None
            };
        } else {
            node.display = Display::Flex;
        }
        let label = confirm_dialog_button_label(dialog, *action, &i18n);
        let width = confirm_dialog_button_width(&label);
        node.width = Val::Px(width);
        node.min_width = Val::Px(width);
        node.flex_grow = 0.0;
        for child in children.iter() {
            if let Ok(mut text) = texts.p1().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

fn confirm_dialog_actions_row_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(40.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(8.0),
        }
        BackgroundColor(Color::NONE)
    }
}

fn spawn_confirm_dialog_button(
    parent: &mut ChildSpawnerCommands,
    action: ConfirmDialogAction,
    text_key: &'static str,
) {
    parent
        .spawn((Button, HoverButton, action))
        .observe(confirm_dialog_actions)
        .queue_apply_scene(confirm_dialog_button_visual_scene())
        .queue_spawn_related_scenes::<Children>(confirm_dialog_button_label_scene(text_key));
}

fn confirm_dialog_button_visual_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            height: Val::Px(default_button_size(34.0)),
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

fn confirm_dialog_button_label_scene(text_key: &'static str) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({text_key})
            TextFont {
                font_size: {default_font_size(15.0)}
            }
            TextColor(Color::WHITE)
            LocalizedText {
                key: {text_key}
            }
            Pickable::IGNORE
        )
    }
}

fn confirm_dialog_button_label(
    dialog: &ConfirmDialogState,
    action: ConfirmDialogAction,
    i18n: &I18n,
) -> String {
    match action {
        ConfirmDialogAction::Primary => i18n.text(dialog.primary_key),
        ConfirmDialogAction::Secondary => dialog
            .secondary_key
            .map(|key| i18n.text(key))
            .unwrap_or_default(),
        ConfirmDialogAction::Cancel => i18n.text(dialog.cancel_key),
    }
}

fn confirm_dialog_message(message: &ConfirmDialogMessage, i18n: &I18n) -> String {
    match message {
        ConfirmDialogMessage::TextKey(key) => i18n.text(key),
        ConfirmDialogMessage::Named { key, name } => i18n.fmt(key, &[("name", name.clone())]),
    }
}

fn confirm_dialog_button_width(label: &str) -> f32 {
    let char_count = label.chars().count() as f32;
    let wide_count = label.chars().filter(|ch| !ch.is_ascii()).count() as f32;
    let estimated_text_width = char_count * 10.0 + wide_count * 8.0;
    estimated_text_width.clamp(118.0, 230.0)
}
