use bevy::prelude::*;

use crate::game::ui::components::{
    default_button_size, full_width_button, label_text, panel_bundle, panel_content,
    panel_title_bar, panel_title_label, text, transparent_node, STATUS_TEXT,
};
use crate::game::ui::types::{
    ConfirmDialogAction, ConfirmDialogMessage, ConfirmDialogState, PanelText, PanelTextKind,
    PanelVisibility, UiRuntime,
};
use crate::shared::i18n::I18n;

pub fn spawn_confirm_dialog(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle(620.0),
        GlobalZIndex(0),
        PanelVisibility::ConfirmDialog,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((
                panel_title_label("", 24.0),
                PanelText(PanelTextKind::ConfirmTitle),
            ));
        });
        panel.spawn(panel_content()).with_children(|panel| {
            panel.spawn((
                text("", 15.0, STATUS_TEXT),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    min_height: Val::Px(54.0),
                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                PanelText(PanelTextKind::ConfirmMessage),
            ));
            panel
                .spawn(confirm_dialog_actions_row())
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
                    spawn_confirm_dialog_button(row, ConfirmDialogAction::Cancel, "button.cancel");
                });
        });
    });
}

pub fn update_confirm_dialog_ui(
    ui_runtime: Res<UiRuntime>,
    i18n: Res<I18n>,
    mut texts: ParamSet<(
        Query<(&PanelText, &mut Text)>,
        Query<&mut Text, Without<PanelText>>,
    )>,
    mut action_buttons: Query<(&ConfirmDialogAction, &mut Node, &Children), With<Button>>,
) {
    if !ui_runtime.is_changed() && !i18n.is_changed() {
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

fn confirm_dialog_actions_row() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Px(default_button_size(40.0)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(8.0),
        ..default()
    })
}

fn spawn_confirm_dialog_button(
    parent: &mut ChildSpawnerCommands,
    action: ConfirmDialogAction,
    text_key: &'static str,
) {
    parent
        .spawn((full_width_button(34.0), action))
        .with_children(|button| {
            button.spawn((
                label_text(text_key, 15.0, Color::WHITE),
                crate::game::ui::types::LocalizedText { key: text_key },
            ));
        });
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
