use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::UiPanelId;
use crate::game::block_editing::BlockPanelTextKind;

const INLINE_TEXT_MAX_LEN: usize = 24;

#[derive(Resource, Default)]
pub struct InlineTextEditState {
    pub panel: Option<UiPanelId>,
    pub pos: Option<IVec3>,
    pub field: Option<BlockPanelTextKind>,
    pub buffer: String,
}

impl InlineTextEditState {
    pub fn is_active(&self) -> bool {
        self.panel.is_some()
    }

    pub fn clear(&mut self) {
        self.panel = None;
        self.pos = None;
        self.field = None;
        self.buffer.clear();
    }

    pub fn start(
        &mut self,
        panel: UiPanelId,
        pos: IVec3,
        field: BlockPanelTextKind,
        initial: String,
    ) {
        self.panel = Some(panel);
        self.pos = Some(pos);
        self.field = Some(field);
        self.buffer = initial.chars().take(INLINE_TEXT_MAX_LEN).collect();
    }
}

pub fn primary_click(click: &mut On<Pointer<Click>>) -> bool {
    click.event.button == PointerButton::Primary
}

pub fn push_text_input(buffer: &mut String, event: &KeyboardInput) {
    let Some(text) = event.text.as_deref() else {
        return;
    };
    for ch in text.chars() {
        push_inline_char(buffer, ch);
    }
}

fn push_inline_char(buffer: &mut String, ch: char) {
    if buffer.chars().count() >= INLINE_TEXT_MAX_LEN || ch.is_control() {
        return;
    }
    buffer.push(ch);
}

pub struct InlineTextInputResult {
    pub confirm: bool,
    pub cancel: bool,
}

pub fn read_inline_text_input(
    keyboard_input: &mut MessageReader<KeyboardInput>,
    buffer: &mut String,
) -> InlineTextInputResult {
    let mut confirm = false;
    let mut cancel = false;
    for event in keyboard_input.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Enter => confirm = true,
            Key::Escape => cancel = true,
            Key::Backspace => {
                buffer.pop();
            }
            _ => push_text_input(buffer, event),
        }
    }
    InlineTextInputResult { confirm, cancel }
}
