use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

/// 方块面板内联编辑状态（仅追踪是否在编辑，输入由 TextPrompt / EditableText 负责）
#[derive(Resource, Default)]
pub struct InlineTextEditState {
    pub panel: Option<crate::game::state::UiPanelId>,
    pub pos: Option<IVec3>,
    pub field: Option<&'static str>,
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
}

pub fn primary_click(click: &mut On<Pointer<Click>>) -> bool {
    click.event.button == PointerButton::Primary
}
