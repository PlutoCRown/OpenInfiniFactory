use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::text::{EditableText, TextEdit};

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

#[derive(Component, Default, Clone)]
pub struct TextPromptRoot;

/// 文本提示框标题
#[derive(Component, Default, Clone)]
pub struct TextPromptTitle;

/// 文本提示框的可编辑输入
#[derive(Component, Default, Clone)]
pub struct TextPromptInput;

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
    /// 打开时需要把默认值写入 EditableText
    seed_input: bool,
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
        self.seed_input = true;
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

fn input_value(inputs: &Query<&EditableText, With<TextPromptInput>>, fallback: &str) -> String {
    inputs
        .iter()
        .next()
        .map(|text| text.value().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

pub fn emit_text_prompt_actions(
    mut click: On<Pointer<Click>>,
    prompt: Res<TextPromptState>,
    host: Res<UiHost>,
    mut actions: MessageWriter<UiAction>,
    buttons: Query<&TextPromptButtonId>,
    inputs: Query<&EditableText, With<TextPromptInput>>,
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
            value: input_value(&inputs, &prompt.value),
        },
        TextPromptButtonId::Cancel => UiActionKind::TextPromptCancel,
    };
    actions.write(UiAction { instance, kind });
}

/// Enter 提交 / Escape 取消（EditableText 负责打字）
pub fn text_prompt_hotkeys(
    mut prompt: ResMut<TextPromptState>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    host: Res<UiHost>,
    mut actions: MessageWriter<UiAction>,
    inputs: Query<&EditableText, With<TextPromptInput>>,
) {
    if !prompt.is_open() {
        return;
    }
    let Some(instance) = host.active_text_prompt_instance() else {
        return;
    };
    if inputs.iter().any(EditableText::is_composing) {
        return;
    }
    let mut submit = false;
    let mut cancel = false;
    for event in keyboard_input.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Enter => submit = true,
            Key::Escape => cancel = true,
            _ => {}
        }
    }
    if submit {
        let value = input_value(&inputs, &prompt.value);
        prompt.value.clone_from(&value);
        actions.write(UiAction {
            instance,
            kind: UiActionKind::TextPromptSubmit { value },
        });
    } else if cancel {
        actions.write(UiAction {
            instance,
            kind: UiActionKind::TextPromptCancel,
        });
    }
}

pub fn update_text_prompt_ui(
    mut prompt: ResMut<TextPromptState>,
    mut focus: ResMut<InputFocus>,
    mut roots: Query<(&mut Node, &mut Visibility), With<TextPromptRoot>>,
    mut titles: Query<&mut Text, With<TextPromptTitle>>,
    mut inputs: Query<(Entity, &mut EditableText), With<TextPromptInput>>,
    buttons: Query<(&TextPromptButtonId, &Children)>,
    mut button_labels: Query<&mut Text, Without<TextPromptTitle>>,
) {
    let visible = prompt.is_open();
    let next_display = if visible {
        Display::Flex
    } else {
        Display::None
    };
    let next_visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for (mut node, mut visibility) in &mut roots {
        if node.display != next_display {
            node.display = next_display;
        }
        visibility.set_if_neq(next_visibility);
    }
    if !visible {
        return;
    }

    if prompt.is_changed() {
        for mut text in &mut titles {
            text.0 = prompt.title.clone();
        }
        let mut labels = Vec::new();
        for (button, children) in &buttons {
            let label = match button {
                TextPromptButtonId::Save => prompt.save_text.clone(),
                TextPromptButtonId::Cancel => prompt.cancel_text.clone(),
            };
            labels.push((children.iter().collect::<Vec<_>>(), label));
        }
        for (children, label) in labels {
            for child in children {
                if let Ok(mut text) = button_labels.get_mut(child) {
                    text.0 = label.clone();
                }
            }
        }
    }

    if prompt.seed_input {
        prompt.seed_input = false;
        let value = prompt.value.clone();
        for (entity, mut editable) in &mut inputs {
            editable.clear();
            editable.max_characters = Some(TEXT_PROMPT_MAX_LEN);
            editable.allow_newlines = false;
            editable.editor.set_text(&value);
            editable.queue_edit(TextEdit::TextEnd(false));
            focus.set(entity, FocusCause::Navigated);
        }
    }
}
