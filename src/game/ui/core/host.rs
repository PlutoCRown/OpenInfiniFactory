use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::state::UiPanelId;
use crate::game::ui::core::confirm_dialog::{
    ConfirmDialogState, ConfirmProps, ConfirmResult, PendingConfirmHandler, spawn_confirm_dialog,
};
use crate::game::ui::core::runtime::{UiModal, UiNavigation};
use crate::game::ui::core::text_prompt::{
    PendingTextPromptHandler, TextPromptProps, TextPromptResult, TextPromptState, spawn_text_prompt,
};
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
    pub const SAVE_SETTINGS: Self = Self(u64::MAX - 3);
    pub const INVENTORY: Self = Self(u64::MAX - 4);
    pub const PAUSE_MENU: Self = Self(u64::MAX - 5);
}

#[derive(Clone)]
pub enum ViewSpec {
    Confirm(ConfirmProps),
    TextPrompt(TextPromptProps),
}

#[derive(Clone, Debug, Eq, Message, PartialEq)]
pub struct UiAction<T: Send + Sync + 'static = UiActionKind> {
    pub instance: UiInstanceId,
    pub kind: T,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiActionKind {
    ConfirmDialog(super::confirm_dialog::ConfirmButtonId),
    TextPromptSubmit { value: String },
    TextPromptCancel,
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

/// 面板挂载策略由功能层提供，宿主只保存实例和实体。
pub struct PanelMount {
    pub panel: UiPanelId,
    pub instance: Option<UiInstanceId>,
    pub replaces: fn(UiPanelId) -> bool,
}

#[derive(Resource, Default)]
pub struct UiHost {
    next_instance: u64,
    stack: Vec<(UiInstanceId, MountedView)>,
}

#[derive(SystemParam)]
pub(crate) struct UiHostCommands<'w> {
    pub host: ResMut<'w, UiHost>,
    pub navigation: ResMut<'w, UiNavigation>,
    pub confirm_dialog: ResMut<'w, ConfirmDialogState>,
    pub text_prompt: ResMut<'w, TextPromptState>,
    pub confirm_pending: NonSendMut<'w, PendingConfirmHandler>,
    pub text_prompt_pending: NonSendMut<'w, PendingTextPromptHandler>,
    pub playing_ui_root: Option<Res<'w, PlayingUiRootEntity>>,
    pub ui_root: Option<Res<'w, UiRootEntity>>,
}

impl UiHostCommands<'_> {
    /// 用功能层构建回调挂载面板，宿主不接收页面业务资源。
    pub fn mount_panel(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        mount: PanelMount,
        navigate: impl FnOnce(&mut UiNavigation),
        build: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> UiInstanceId {
        self.host
            .mount_panel(commands, root, &mut self.navigation, mount, navigate, build)
    }

    fn active_root(&self) -> Option<Entity> {
        self.playing_ui_root
            .as_ref()
            .map(|root| root.0)
            .or_else(|| self.ui_root.as_ref().map(|root| root.0))
    }

    pub fn unmount_panel(&mut self, panel: UiPanelId, commands: &mut Commands) {
        self.host
            .unmount_panel(panel, &mut self.navigation, Some(commands));
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
            &mut self.navigation,
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
            &mut self.navigation,
            props,
            &mut self.confirm_dialog,
            &mut self.text_prompt,
            &mut self.text_prompt_pending,
            on_complete,
        )
    }
}

impl UiHost {
    /// 通用面板挂载：替换指定实例，功能层负责导航意图和子树内容。
    pub fn mount_panel(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        navigation: &mut UiNavigation,
        mount: PanelMount,
        navigate: impl FnOnce(&mut UiNavigation),
        build: impl FnOnce(&mut ChildSpawnerCommands),
    ) -> UiInstanceId {
        let replaced: Vec<_> = self
            .stack
            .iter()
            .filter_map(|(_, view)| match view {
                MountedView::Panel { panel, .. } if (mount.replaces)(*panel) => Some(*panel),
                _ => None,
            })
            .collect();
        for panel in replaced {
            self.unmount_panel(panel, navigation, Some(commands));
        }
        navigate(navigation);
        let id = mount.instance.unwrap_or_else(|| self.next_id());
        let entity = root.map(|root| {
            let container = commands
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
                .with_children(build)
                .id();
            commands.entity(root).add_child(container);
            container
        });
        self.stack.push((
            id,
            MountedView::Panel {
                panel: mount.panel,
                entity,
            },
        ));
        id
    }

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
        matches!(
            id,
            UiInstanceId::START_MENU
                | UiInstanceId::SAVE_LIST
                | UiInstanceId::INVENTORY
                | UiInstanceId::PAUSE_MENU
        ) || self.stack.iter().any(|(instance, _)| *instance == id)
    }

    pub fn mount(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        navigation: &mut UiNavigation,
        spec: ViewSpec,
        confirm_dialog: &mut ConfirmDialogState,
        text_prompt: &mut TextPromptState,
    ) -> UiInstanceId {
        self.despawn_modals(commands);
        let id = self.next_id();
        match spec {
            ViewSpec::Confirm(props) => {
                navigation.open_modal(UiModal::Confirm);
                confirm_dialog.reset_for_open(props);
                let entity = spawn_modal_child(commands, root, spawn_confirm_dialog);
                self.push_modal(id, MountedView::Confirm { entity });
            }
            ViewSpec::TextPrompt(props) => {
                navigation.open_modal(UiModal::TextPrompt);
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
        runtime: &mut UiNavigation,
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
                runtime.close_modal();
                if let (Some(commands), Some(entity)) = (commands, entity) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    pub fn unmount_panel(
        &mut self,
        panel: UiPanelId,
        runtime: &mut UiNavigation,
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
        runtime: &mut UiNavigation,
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
        runtime.close_all_panels();
    }

    /// 按需挂载方块属性面板（含下拉 overlay）

    pub fn dispatch_completions(
        &mut self,
        navigation: &mut UiNavigation,
        confirm_dialog: &mut ConfirmDialogState,
        confirm_pending: &mut PendingConfirmHandler,
        text_prompt: &mut TextPromptState,
        text_prompt_pending: &mut PendingTextPromptHandler,
        commands: &mut Commands,
    ) {
        self.complete_confirm(navigation, confirm_dialog, confirm_pending, commands);
        self.complete_text_prompt(navigation, text_prompt, text_prompt_pending, commands);
    }

    pub fn open_confirm_then(
        &mut self,
        commands: &mut Commands,
        root: Option<Entity>,
        navigation: &mut UiNavigation,
        props: ConfirmProps,
        confirm_dialog: &mut ConfirmDialogState,
        text_prompt: &mut TextPromptState,
        pending: &mut PendingConfirmHandler,
        on_complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        let id = self.mount(
            commands,
            root,
            navigation,
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
        navigation: &mut UiNavigation,
        props: TextPromptProps,
        confirm_dialog: &mut ConfirmDialogState,
        text_prompt: &mut TextPromptState,
        pending: &mut PendingTextPromptHandler,
        on_complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        let id = self.mount(
            commands,
            root,
            navigation,
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
        navigation: &mut UiNavigation,
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
        navigation.close_modal();
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
        navigation: &mut UiNavigation,
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
        navigation.close_modal();
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
    mut navigation: ResMut<UiNavigation>,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    mut confirm_pending: NonSendMut<PendingConfirmHandler>,
    mut text_prompt: ResMut<TextPromptState>,
    mut text_prompt_pending: NonSendMut<PendingTextPromptHandler>,
    mut commands: Commands,
) {
    host.dispatch_completions(
        &mut navigation,
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
        }
    }
}
