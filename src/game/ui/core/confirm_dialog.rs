use bevy::ecs::system::SystemParam;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::ui::core::text_input::primary_click;

type ConfirmHandler = Box<dyn FnOnce(ConfirmResult, &mut World) + Send>;

/// Stored as a [`NonSend`] resource because [`ConfirmHandler`] is not [`Sync`].
#[derive(Default)]
pub(crate) struct PendingConfirmHandler {
    pub handler: Option<ConfirmHandler>,
}

#[derive(SystemParam)]
pub struct ActiveConfirmDialog<'w> {
    pub dialog: ResMut<'w, ConfirmDialogState>,
    pub pending: NonSendMut<'w, PendingConfirmHandler>,
}

impl ActiveConfirmDialog<'_> {
    /// Opens the dialog and runs `on_complete` once the user closes it.
    ///
    /// Bevy 主循环是同步的，无法真正 `.await` 用户点击；这个 API 在**同一次调用**里
    /// 注册关闭后的逻辑，语义上等价于：
    ///
    /// ```ignore
    /// let result = dialog.open(spec).await;
    /// match result { ... }
    /// ```
    pub fn open_then(
        &mut self,
        spec: ConfirmOpen,
        on_complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) {
        self.dialog.reset_for_open(spec);
        self.pending.handler = Some(Box::new(on_complete));
    }
}

#[derive(Component)]
pub struct ConfirmTitleText;

#[derive(Component)]
pub struct ConfirmMessageText;

/// Which physical button was pressed — not a business outcome.
#[derive(Component, Clone, Copy)]
pub enum ConfirmButtonId {
    Confirm,
    Extra,
    Cancel,
}

#[derive(Clone)]
pub struct ConfirmExtraButton {
    pub text: String,
    pub tag: u32,
}

#[derive(Clone)]
pub struct ConfirmOpen {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub extra: Option<ConfirmExtraButton>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmResult {
    Confirmed,
    Cancelled,
    Extra(u32),
}

#[derive(Resource, Default)]
pub struct ConfirmDialogState {
    open: bool,
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub extra: Option<ConfirmExtraButton>,
    result: Option<ConfirmResult>,
}

impl ConfirmDialogState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Shows the dialog shell without a completion handler.
    #[allow(dead_code)]
    pub fn open(&mut self, spec: ConfirmOpen) {
        self.reset_for_open(spec);
    }

    pub(crate) fn reset_for_open(&mut self, spec: ConfirmOpen) {
        self.open = true;
        self.title = spec.title;
        self.message = spec.message;
        self.confirm_text = spec.confirm_text;
        self.cancel_text = spec.cancel_text;
        self.extra = spec.extra;
        self.result = None;
    }

    pub fn resolve(&mut self, button: ConfirmButtonId) {
        if !self.open {
            return;
        }
        self.result = Some(match button {
            ConfirmButtonId::Confirm => ConfirmResult::Confirmed,
            ConfirmButtonId::Cancel => ConfirmResult::Cancelled,
            ConfirmButtonId::Extra => {
                ConfirmResult::Extra(self.extra.as_ref().map(|extra| extra.tag).unwrap_or(0))
            }
        });
        self.open = false;
    }

    pub fn take_result(&mut self) -> Option<ConfirmResult> {
        self.result.take()
    }
}

/// Runs [`ConfirmDialogState::open_then`] callbacks after the user closes the dialog.
pub fn dispatch_confirm_completion(
    mut dialog: ResMut<ConfirmDialogState>,
    mut pending: NonSendMut<PendingConfirmHandler>,
    mut commands: Commands,
) {
    if pending.handler.is_none() {
        return;
    }
    let Some(result) = dialog.take_result() else {
        return;
    };
    let Some(handler) = pending.handler.take() else {
        return;
    };
    commands.queue(move |world: &mut World| {
        handler(result, world);
    });
}

pub fn confirm_dialog_clicks(
    mut click: On<Pointer<Click>>,
    mut dialog: ResMut<ConfirmDialogState>,
    buttons: Query<&ConfirmButtonId>,
) {
    if !primary_click(&mut click) || !dialog.is_open() {
        return;
    }
    let Ok(button) = buttons.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    dialog.resolve(button);
}

pub fn update_confirm_dialog_ui(
    dialog: Res<ConfirmDialogState>,
    mut texts: ParamSet<(
        Query<&mut Text, With<ConfirmTitleText>>,
        Query<&mut Text, With<ConfirmMessageText>>,
        Query<&mut Text, (Without<ConfirmTitleText>, Without<ConfirmMessageText>)>,
    )>,
    mut action_buttons: Query<(&ConfirmButtonId, &mut Node, &Children), With<Button>>,
) {
    if !dialog.is_open() {
        return;
    }
    if !dialog.is_changed() {
        return;
    }

    for mut text in &mut texts.p0() {
        text.0 = dialog.title.clone();
    }
    for mut text in &mut texts.p1() {
        text.0 = dialog.message.clone();
    }

    let mut button_labels = Vec::new();
    for (button, mut node, children) in &mut action_buttons {
        let (visible, label) = match button {
            ConfirmButtonId::Confirm => (true, dialog.confirm_text.clone()),
            ConfirmButtonId::Cancel => (true, dialog.cancel_text.clone()),
            ConfirmButtonId::Extra => dialog
                .extra
                .as_ref()
                .map(|extra| (true, extra.text.clone()))
                .unwrap_or((false, String::new())),
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
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
