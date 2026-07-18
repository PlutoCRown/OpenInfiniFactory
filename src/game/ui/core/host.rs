use bevy::ecs::system::SystemParam;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

use crate::game::blocks::panels::find_block_panel_hooks;
use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::core::confirm_dialog::{
    ConfirmDialogState, ConfirmProps, ConfirmResult, PendingConfirmHandler, spawn_confirm_dialog,
};
use crate::game::ui::core::runtime::{UiPanelContext, UiRuntime};
use crate::game::ui::core::text_prompt::{
    PendingTextPromptHandler, TextPromptProps, TextPromptResult, TextPromptState, spawn_text_prompt,
};
use crate::game::ui::features::save::types::SaveListAction;
use crate::game::ui::features::settings::types::SettingsAction;
use crate::game::ui::screens::spawn_settings_panel;
use crate::game::ui::types::InventorySlot;
#[derive(Resource, Clone, Copy)]
pub struct UiRootEntity(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct PlayingUiRootEntity(pub Entity);

#[derive(Component)]
pub struct UiHostMountRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UiInstanceId(u64);

impl UiInstanceId {
    pub const START_MENU: Self = Self(u64::MAX);
    pub const SAVE_LIST: Self = Self(u64::MAX - 1);
    pub const SETTINGS: Self = Self(u64::MAX - 2);
    pub const INVENTORY: Self = Self(u64::MAX - 4);
    pub const PAUSE_MENU: Self = Self(u64::MAX - 5);
}

#[derive(Clone)]
pub enum ViewSpec {
    Confirm(ConfirmProps),
    TextPrompt(TextPromptProps),
}

#[derive(Clone, Debug, Eq, Message, PartialEq)]
pub struct UiAction {
    pub instance: UiInstanceId,
    pub kind: UiActionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiActionKind {
    SaveList(SaveListAction),
    Settings(SettingsAction),
    InventorySlot {
        slot: InventorySlot,
        button: PointerButton,
    },
    ConfirmDialog(super::confirm_dialog::ConfirmButtonId),
    TextPromptSubmit {
        value: String,
    },
    TextPromptCancel,
    PanelClose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MountedView {
    Confirm {
        entity: Option<Entity>,
    },
    TextPrompt {
        entity: Option<Entity>,
    },
    Panel {
        panel: UiPanelId,
        entity: Option<Entity>,
    },
}

#[derive(Resource, Default)]
pub struct UiHost {
    next_instance: u64,
    stack: Vec<(UiInstanceId, MountedView)>,
}

#[derive(SystemParam)]
pub(crate) struct UiHostCommands<'w> {
    pub host: ResMut<'w, UiHost>,
    pub runtime: ResMut<'w, UiRuntime>,
    pub confirm_dialog: ResMut<'w, ConfirmDialogState>,
    pub text_prompt: ResMut<'w, TextPromptState>,
    pub confirm_pending: NonSendMut<'w, PendingConfirmHandler>,
    pub text_prompt_pending: NonSendMut<'w, PendingTextPromptHandler>,
    pub playing_ui_root: Option<Res<'w, PlayingUiRootEntity>>,
    pub ui_root: Option<Res<'w, UiRootEntity>>,
}

impl UiHostCommands<'_> {
    fn active_root(&self) -> Option<Entity> {
        self.playing_ui_root
            .as_ref()
            .map(|root| root.0)
            .or_else(|| self.ui_root.as_ref().map(|root| root.0))
    }

    pub fn mount_settings(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        context: UiPanelContext,
        settings: &GameSettings,
    ) -> UiInstanceId {
        self.host
            .mount_settings(commands, root, &mut self.runtime, context, settings)
    }

    pub fn unmount_panel(&mut self, panel: UiPanelId, commands: &mut Commands) {
        self.host
            .unmount_panel(panel, &mut self.runtime, Some(commands));
    }

    pub fn mount_block_panel(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        panel: UiPanelId,
        pos: IVec3,
    ) -> UiInstanceId {
        self.host
            .mount_block_panel(commands, root, &mut self.runtime, panel, pos)
    }

    pub fn open_confirm_then(
        &mut self,
        commands: &mut Commands,
        props: ConfirmProps,
        on_complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        let root = self.active_root();
        self.host.open_confirm_then(
            commands,
            root,
            props,
            &mut self.confirm_dialog,
            &mut self.text_prompt,
            &mut self.confirm_pending,
            on_complete,
        )
    }

    pub fn open_text_prompt_then(
        &mut self,
        commands: &mut Commands,
        props: TextPromptProps,
        on_complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        let root = self.active_root();
        self.host.open_text_prompt_then(
            commands,
            root,
            props,
            &mut self.confirm_dialog,
            &mut self.text_prompt,
            &mut self.text_prompt_pending,
            on_complete,
        )
    }
}

impl UiHost {
    pub fn active_confirm_instance(&self) -> Option<UiInstanceId> {
        self.stack.iter().rev().find_map(|(instance, view)| {
            matches!(view, MountedView::Confirm { .. }).then_some(*instance)
        })
    }

    pub fn active_text_prompt_instance(&self) -> Option<UiInstanceId> {
        self.stack.iter().rev().find_map(|(instance, view)| {
            matches!(view, MountedView::TextPrompt { .. }).then_some(*instance)
        })
    }

    pub fn has_instance(&self, id: UiInstanceId) -> bool {
        self.stack.iter().any(|(instance, _)| *instance == id)
    }

    pub fn mount(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        spec: ViewSpec,
        confirm_dialog: &mut ConfirmDialogState,
        text_prompt: &mut TextPromptState,
    ) -> UiInstanceId {
        self.despawn_modals(commands);
        let id = self.next_id();
        match spec {
            ViewSpec::Confirm(props) => {
                confirm_dialog.reset_for_open(props);
                let entity = spawn_modal_child(commands, root, spawn_confirm_dialog);
                self.push_modal(id, MountedView::Confirm { entity });
            }
            ViewSpec::TextPrompt(props) => {
                text_prompt.reset_for_open(props);
                let entity = spawn_modal_child(commands, root, spawn_text_prompt);
                self.push_modal(id, MountedView::TextPrompt { entity });
            }
        }
        id
    }

    pub fn unmount(
        &mut self,
        id: UiInstanceId,
        runtime: &mut UiRuntime,
        commands: Option<&mut Commands>,
    ) {
        let Some(index) = self.stack.iter().position(|(instance, _)| *instance == id) else {
            return;
        };
        let (_, view) = self.stack.remove(index);
        match view {
            MountedView::Panel { panel, entity } => {
                runtime.close_panel(panel);
                if let (Some(commands), Some(entity)) = (commands, entity) {
                    commands.entity(entity).despawn();
                }
            }
            MountedView::Confirm { entity } | MountedView::TextPrompt { entity } => {
                if let (Some(commands), Some(entity)) = (commands, entity) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    pub fn unmount_panel(
        &mut self,
        panel: UiPanelId,
        runtime: &mut UiRuntime,
        commands: Option<&mut Commands>,
    ) {
        if let Some(index) = self
            .stack
            .iter()
            .position(|(_, view)| matches!(view, MountedView::Panel { panel: mounted, .. } if *mounted == panel))
        {
            let (id, _) = self.stack[index];
            self.unmount(id, runtime, commands);
        } else {
            runtime.close_panel(panel);
        }
    }

    /// 卸掉所有已挂载面板（退出 Playing 时用，可不传 commands）
    pub fn unmount_all_panels(
        &mut self,
        runtime: &mut UiRuntime,
        mut commands: Option<&mut Commands>,
    ) {
        let panels: Vec<UiPanelId> = self
            .stack
            .iter()
            .filter_map(|(_, view)| match view {
                MountedView::Panel { panel, .. } => Some(*panel),
                _ => None,
            })
            .collect();
        for panel in panels {
            self.unmount_panel(panel, runtime, commands.as_deref_mut());
        }
    }

    /// 按需挂载方块属性面板（含下拉 overlay）
    pub fn mount_block_panel(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        runtime: &mut UiRuntime,
        panel: UiPanelId,
        pos: IVec3,
    ) -> UiInstanceId {
        let other_blocks: Vec<UiPanelId> = self
            .stack
            .iter()
            .filter_map(|(_, view)| match view {
                MountedView::Panel { panel: mounted, .. } if !mounted.is_settings() => {
                    Some(*mounted)
                }
                _ => None,
            })
            .collect();
        for mounted in other_blocks {
            self.unmount_panel(mounted, runtime, Some(commands));
        }

        let id = self.next_id();
        runtime.open_block(panel, pos);
        let Some(hooks) = find_block_panel_hooks(panel) else {
            self.stack.push((
                id,
                MountedView::Panel {
                    panel,
                    entity: None,
                },
            ));
            return id;
        };

        let entity = root.map(|root| {
            let mut container = None;
            commands.entity(root).with_children(|root| {
                let spawned = root
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        UiHostMountRoot,
                        Pickable::IGNORE,
                    ))
                    .with_children(|container| {
                        (hooks.spawn_panel)(container);
                        (hooks.spawn_overlays)(container);
                    })
                    .id();
                container = Some(spawned);
            });
            container.unwrap_or(root)
        });
        self.stack.push((id, MountedView::Panel { panel, entity }));
        id
    }

    pub fn mount_settings(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        runtime: &mut UiRuntime,
        context: UiPanelContext,
        settings: &GameSettings,
    ) -> UiInstanceId {
        let id = self.next_id();
        self.unmount_panel(UiPanelId::Settings, runtime, Some(commands));
        runtime.open(UiPanelId::Settings, context);
        let entity = root.map(|root| {
            let mut container = None;
            commands.entity(root).with_children(|root| {
                let spawned = root
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        UiHostMountRoot,
                    ))
                    .with_children(|container| {
                        spawn_settings_panel(container, settings);
                    })
                    .id();
                container = Some(spawned);
            });
            container.unwrap_or(root)
        });
        self.stack.push((
            id,
            MountedView::Panel {
                panel: UiPanelId::Settings,
                entity,
            },
        ));
        id
    }

    pub fn modal_open(&self) -> bool {
        self.stack.iter().any(|(_, view)| {
            matches!(
                view,
                MountedView::Confirm { .. } | MountedView::TextPrompt { .. }
            )
        })
    }

    pub fn confirm_open(&self) -> bool {
        self.stack
            .iter()
            .any(|(_, view)| matches!(view, MountedView::Confirm { .. }))
    }

    pub fn dispatch_completions(
        &mut self,
        confirm_dialog: &mut ConfirmDialogState,
        confirm_pending: &mut PendingConfirmHandler,
        text_prompt: &mut TextPromptState,
        text_prompt_pending: &mut PendingTextPromptHandler,
        commands: &mut Commands,
    ) {
        self.complete_confirm(confirm_dialog, confirm_pending, commands);
        self.complete_text_prompt(text_prompt, text_prompt_pending, commands);
    }

    pub fn open_confirm_then(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        props: ConfirmProps,
        confirm_dialog: &mut ConfirmDialogState,
        text_prompt: &mut TextPromptState,
        pending: &mut PendingConfirmHandler,
        on_complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        let id = self.mount(
            commands,
            root,
            ViewSpec::Confirm(props),
            confirm_dialog,
            text_prompt,
        );
        pending.handler = Some(Box::new(on_complete));
        id
    }

    pub fn open_text_prompt_then(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        props: TextPromptProps,
        confirm_dialog: &mut ConfirmDialogState,
        text_prompt: &mut TextPromptState,
        pending: &mut PendingTextPromptHandler,
        on_complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        let id = self.mount(
            commands,
            root,
            ViewSpec::TextPrompt(props),
            confirm_dialog,
            text_prompt,
        );
        pending.handler = Some(Box::new(on_complete));
        id
    }

    fn next_id(&mut self) -> UiInstanceId {
        let id = UiInstanceId(self.next_instance);
        self.next_instance = self.next_instance.wrapping_add(1);
        id
    }

    fn push_modal(&mut self, id: UiInstanceId, view: MountedView) {
        self.stack.retain(|(_, mounted)| {
            !matches!(
                mounted,
                MountedView::Confirm { .. } | MountedView::TextPrompt { .. }
            )
        });
        self.stack.push((id, view));
    }

    fn despawn_modals(&mut self, commands: &mut Commands) {
        let entities: Vec<Entity> = self
            .stack
            .iter()
            .filter_map(|(_, view)| match view {
                MountedView::Confirm { entity } | MountedView::TextPrompt { entity } => *entity,
                _ => None,
            })
            .collect();
        self.stack.retain(|(_, mounted)| {
            !matches!(
                mounted,
                MountedView::Confirm { .. } | MountedView::TextPrompt { .. }
            )
        });
        for entity in entities {
            commands.entity(entity).despawn();
        }
    }

    fn complete_confirm(
        &mut self,
        confirm_dialog: &mut ConfirmDialogState,
        pending: &mut PendingConfirmHandler,
        commands: &mut Commands,
    ) {
        if pending.handler.is_none() {
            return;
        }
        let Some(result) = confirm_dialog.take_result() else {
            return;
        };
        let entity = self.stack.iter().find_map(|(_, view)| match view {
            MountedView::Confirm { entity } => *entity,
            _ => None,
        });
        self.stack
            .retain(|(_, view)| !matches!(view, MountedView::Confirm { .. }));
        if let Some(entity) = entity {
            commands.entity(entity).despawn();
        }
        let Some(handler) = pending.handler.take() else {
            return;
        };
        commands.queue(move |world: &mut World| {
            handler(result, world);
        });
    }

    fn complete_text_prompt(
        &mut self,
        text_prompt: &mut TextPromptState,
        pending: &mut PendingTextPromptHandler,
        commands: &mut Commands,
    ) {
        if pending.handler.is_none() {
            return;
        }
        let Some(result) = text_prompt.take_result() else {
            return;
        };
        let entity = self.stack.iter().find_map(|(_, view)| match view {
            MountedView::TextPrompt { entity } => *entity,
            _ => None,
        });
        self.stack
            .retain(|(_, view)| !matches!(view, MountedView::TextPrompt { .. }));
        if let Some(entity) = entity {
            commands.entity(entity).despawn();
        }
        let Some(handler) = pending.handler.take() else {
            return;
        };
        commands.queue(move |world: &mut World| {
            handler(result, world);
        });
    }
}

fn spawn_modal_child(
    commands: &mut Commands,
    root: Option<Entity>,
    spawn: fn(&mut ChildSpawnerCommands) -> Entity,
) -> Option<Entity> {
    root.map(|root| {
        let mut entity = None;
        commands.entity(root).with_children(|root| {
            entity = Some(spawn(root));
        });
        entity.unwrap_or(root)
    })
}

pub fn dispatch_ui_host_completions(
    mut host: ResMut<UiHost>,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    mut confirm_pending: NonSendMut<PendingConfirmHandler>,
    mut text_prompt: ResMut<TextPromptState>,
    mut text_prompt_pending: NonSendMut<PendingTextPromptHandler>,
    mut commands: Commands,
) {
    host.dispatch_completions(
        &mut confirm_dialog,
        &mut confirm_pending,
        &mut text_prompt,
        &mut text_prompt_pending,
        &mut commands,
    );
}

pub fn dispatch_ui_action(
    host: Res<UiHost>,
    mut actions: MessageReader<UiAction>,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    mut text_prompt: ResMut<TextPromptState>,
) {
    for action in actions.read() {
        if !host.has_instance(action.instance) {
            continue;
        }
        match &action.kind {
            UiActionKind::ConfirmDialog(button) => confirm_dialog.resolve(*button),
            UiActionKind::TextPromptSubmit { value } => {
                text_prompt.value.clone_from(value);
                text_prompt.submit();
            }
            UiActionKind::TextPromptCancel => text_prompt.cancel(),
            UiActionKind::SaveList(_)
            | UiActionKind::Settings(_)
            | UiActionKind::InventorySlot { .. } => {}
            UiActionKind::PanelClose => {}
        }
    }
}
