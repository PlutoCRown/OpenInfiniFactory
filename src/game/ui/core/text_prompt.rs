use bevy::ecs::system::SystemParam;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::ui::core::text_input::primary_click;

const TEXT_PROMPT_MAX_LEN: usize = 24;

type TextPromptHandler = Box<dyn FnOnce(TextPromptResult, &mut World) + Send>;

/// Stored as a [`NonSend`] resource because [`TextPromptHandler`] is not [`Sync`].
#[derive(Default)]
pub(crate) struct PendingTextPromptHandler {
    pub handler: Option<TextPromptHandler>,
}

#[derive(SystemParam)]
pub struct ActiveTextPrompt<'w> {
    pub prompt: ResMut<'w, TextPromptState>,
    pub pending: NonSendMut<'w, PendingTextPromptHandler>,
}

impl ActiveTextPrompt<'_> {
    /// Opens the prompt and runs `on_complete` once the user saves or cancels.
    ///
    /// Bevy 主循环是同步的，无法真正 `.await` 用户输入；语义上等价于
    /// `let result = prompt.open(spec).await; match result { ... }`。
    pub fn open_then(
        &mut self,
        spec: TextPromptOpen,
        on_complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) {
        self.prompt.reset_for_open(spec);
        self.pending.handler = Some(Box::new(on_complete));
    }
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
pub struct TextPromptOpen {
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

    pub(crate) fn reset_for_open(&mut self, spec: TextPromptOpen) {
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

/// Runs [`ActiveTextPrompt::open_then`] callbacks after the user closes the prompt.
pub fn dispatch_text_prompt_completion(
    mut prompt: ResMut<TextPromptState>,
    mut pending: NonSendMut<PendingTextPromptHandler>,
    mut commands: Commands,
) {
    if pending.handler.is_none() {
        return;
    }
    let Some(result) = prompt.take_result() else {
        return;
    };
    let Some(handler) = pending.handler.take() else {
        return;
    };
    commands.queue(move |world: &mut World| {
        handler(result, world);
    });
}

pub fn text_prompt_clicks(
    mut click: On<Pointer<Click>>,
    mut prompt: ResMut<TextPromptState>,
    buttons: Query<&TextPromptButtonId>,
) {
    if !primary_click(&mut click) || !prompt.is_open() {
        return;
    }
    let Ok(button) = buttons.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    match button {
        TextPromptButtonId::Save => prompt.submit(),
        TextPromptButtonId::Cancel => prompt.cancel(),
    }
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
