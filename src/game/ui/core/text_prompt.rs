use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::ui::core::host::{UiAction, UiActionKind, UiHost};
use crate::game::ui::core::text_input::primary_click;

const TEXT_PROMPT_MAX_LEN: usize = 24;

type TextPromptHandler = Box<dyn FnOnce(TextPromptResult, &mut World) + Send>;

/// Stored as a [`NonSend`] resource because [`TextPromptHandler`] is not [`Sync`].
#[derive(Default)]
pub struct PendingTextPromptHandler {
    pub handler: Option<TextPromptHandler>,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TextPromptButtonId {
    Save,
    Cancel,
}

#[derive(Component)]
pub struct TextPromptRoot;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub enum TextPromptText {
    Title,
    Value,
}

#[derive(Clone)]
pub struct TextPromptProps {
    pub title: String,
    pub default_value: String,
    pub save_text: String,
    pub cancel_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextPromptResult {
    Saved(String),
    Cancelled,
}

#[derive(Resource, Default)]
pub struct TextPromptState {
    open: bool,
    pub title: String,
    pub value: String,
    pub save_text: String,
    pub cancel_text: String,
    result: Option<TextPromptResult>,
}

impl TextPromptState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn reset_for_open(&mut self, spec: TextPromptProps) {
        self.open = true;
        self.title = spec.title;
        self.value = spec
            .default_value
            .chars()
            .take(TEXT_PROMPT_MAX_LEN)
            .collect();
        self.save_text = spec.save_text;
        self.cancel_text = spec.cancel_text;
        self.result = None;
    }

    pub fn submit(&mut self) {
        if !self.open {
            return;
        }
        self.result = Some(TextPromptResult::Saved(self.value.clone()));
        self.open = false;
    }

    pub fn cancel(&mut self) {
        if !self.open {
            return;
        }
        self.result = Some(TextPromptResult::Cancelled);
        self.open = false;
    }

    pub fn take_result(&mut self) -> Option<TextPromptResult> {
        self.result.take()
    }
}

pub fn emit_text_prompt_actions(
    mut click: On<Pointer<Click>>,
    prompt: Res<TextPromptState>,
    host: Res<UiHost>,
    mut actions: MessageWriter<UiAction>,
    buttons: Query<&TextPromptButtonId>,
) {
    if !primary_click(&mut click) || !prompt.is_open() {
        return;
    }
    let Ok(button) = buttons.get(click.entity).copied() else {
        return;
    };
    let Some(instance) = host.active_text_prompt_instance() else {
        return;
    };
    click.propagate(false);
    let kind = match button {
        TextPromptButtonId::Save => UiActionKind::TextPromptSubmit {
            value: prompt.value.clone(),
        },
        TextPromptButtonId::Cancel => UiActionKind::TextPromptCancel,
    };
    actions.write(UiAction { instance, kind });
}

pub fn update_text_prompt_ui(
    prompt: Res<TextPromptState>,
    mut roots: Query<(&mut Node, &mut Visibility), With<TextPromptRoot>>,
    mut texts: ParamSet<(
        Query<(&TextPromptText, &mut Text)>,
        Query<(&TextPromptButtonId, &Children)>,
        Query<&mut Text, Without<TextPromptText>>,
    )>,
) {
    let visible = prompt.is_open();
    for (mut node, mut visibility) in &mut roots {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !visible {
        return;
    }
    if !prompt.is_changed() {
        return;
    }

    for (marker, mut text) in &mut texts.p0() {
        text.0 = match marker {
            TextPromptText::Title => prompt.title.clone(),
            TextPromptText::Value => format!("{}_", prompt.value),
        };
    }

    let mut button_labels = Vec::new();
    for (button, children) in &texts.p1() {
        let label = match button {
            TextPromptButtonId::Save => prompt.save_text.clone(),
            TextPromptButtonId::Cancel => prompt.cancel_text.clone(),
        };
        button_labels.push((children.iter().collect::<Vec<_>>(), label));
    }
    for (children, label) in button_labels {
        for child in children {
            if let Ok(mut text) = texts.p2().get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}
