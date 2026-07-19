//! 全局 `ui` / `i18n` 访问层。
//!
//! 由 `bind_ui_scope` / `unbind_ui_scope` 在每帧 UI 处理区间绑定 `World`；
//! 业务代码 `use crate::game::ui::access::{i18n, ui}` 后直接 `i18n.t("key")` / `ui.open_*`，勿再作为参数传递。

use std::cell::Cell;
use std::ptr::NonNull;

use bevy::ecs::system::NonSendMarker;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::World;
use bevy::prelude::*;

/// 声明于使用 `i18n` / `ui` 的系统中，保证其在主线程执行（与 `bind_ui_scope` 同线程）。
pub type UiMainThread = NonSendMarker;

use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::core::confirm_dialog::{ConfirmProps, ConfirmResult};
use crate::game::ui::core::host::{UiHostCommands, UiInstanceId};
use crate::game::ui::core::runtime::UiPanelContext;
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::shared::i18n::{I18n, Language};

thread_local! {
    static UI_WORLD: Cell<Option<NonNull<World>>> = const { Cell::new(None) };
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiAccessScope;

pub fn bind_ui_scope(world: &mut World) {
    UI_WORLD.with(|cell| cell.set(Some(NonNull::from(world))));
}

pub fn unbind_ui_scope(_world: &mut World) {
    UI_WORLD.with(|cell| cell.set(None));
}

fn with_world<R>(f: impl FnOnce(&mut World) -> R) -> R {
    UI_WORLD.with(|cell| {
        let Some(mut ptr) = cell.get() else {
            panic!("ui/i18n used outside UiAccessScope; ensure bind_ui_scope runs first");
        };
        // SAFETY: `bind_ui_scope` runs on the main thread before UI systems in the same
        // frame; pointers are cleared before the next bind. Callers must not hold refs across
        // `unbind_ui_scope`.
        let world = unsafe { ptr.as_mut() };
        f(world)
    })
}

fn with_world_immut<R>(f: impl FnOnce(&World) -> R) -> R {
    UI_WORLD.with(|cell| {
        let Some(ptr) = cell.get() else {
            panic!("ui/i18n used outside UiAccessScope; ensure bind_ui_scope runs first");
        };
        let world = unsafe { ptr.as_ref() };
        f(world)
    })
}

fn with_ui<R>(f: impl FnOnce(&mut UiHostCommands) -> R) -> R {
    with_world(|world| {
        let mut state = SystemState::<UiHostCommands>::new(world);
        let mut params = state.get_mut(world).unwrap();
        f(&mut params)
    })
}

/// 全局 i18n 访问点（spawn/低频）。热路径请用 `Res<I18n>` + `t`/`fmt_into`。
#[allow(non_upper_case_globals)]
pub const i18n: I18nAccess = I18nAccess;

#[derive(Clone, Copy, Debug, Default)]
pub struct I18nAccess;

impl I18nAccess {
    /// 低频/spawn：返回拥有的 String（内部一次 to_owned）
    pub fn t(self, key: &'static str) -> String {
        with_world_immut(|world| world.resource::<I18n>().t(key).to_owned())
    }

    /// 低频：模板替换；热路径用 `I18n::fmt_into`
    pub fn fmt(self, key: &'static str, values: &[(&str, &str)]) -> String {
        with_world_immut(|world| world.resource::<I18n>().fmt(key, values))
    }

    pub fn language(self) -> Language {
        with_world_immut(|world| world.resource::<I18n>().language())
    }
}

/// 全局 UI 访问点。用法：`ui.open_confirm_then(...)`
#[allow(non_upper_case_globals)]
pub const ui: UiAccess = UiAccess;

#[derive(Clone, Copy, Debug, Default)]
pub struct UiAccess;

impl UiAccess {
    pub fn mount_settings(
        self,
        commands: &mut Commands,
        root: Option<Entity>,
        context: UiPanelContext,
    ) -> UiInstanceId {
        with_world(|world| {
            let settings = {
                let settings = world.resource::<GameSettings>();
                GameSettings {
                    fov_degrees: settings.fov_degrees,
                    ui_scale: settings.ui_scale,
                    gravity_scale: settings.gravity_scale,
                    mouse_sensitivity_x: settings.mouse_sensitivity_x,
                    mouse_sensitivity_y: settings.mouse_sensitivity_y,
                }
            };
            let scroll_height = {
                use crate::game::ui::screens::SETTINGS_SCROLL_CHROME;
                use bevy::window::PrimaryWindow;
                let scale = world.resource::<UiScale>().0.max(0.01);
                let window_h = world
                    .query_filtered::<&Window, With<PrimaryWindow>>()
                    .iter(world)
                    .next()
                    .map(|window| window.height())
                    .unwrap_or(720.0);
                (window_h / scale - SETTINGS_SCROLL_CHROME).max(80.0)
            };
            let mut state = SystemState::<UiHostCommands>::new(world);
            let mut params = state.get_mut(world).unwrap();
            params.mount_settings(commands, root, context, &settings, scroll_height)
        })
    }

    pub fn unmount_panel(self, panel: UiPanelId, commands: &mut Commands) {
        with_ui(|host| host.unmount_panel(panel, commands));
    }

    /// 挂载方块面板并立即 flush，使实体同帧可被查询
    pub fn mount_block_panel(
        self,
        root: Option<Entity>,
        panel: UiPanelId,
        pos: IVec3,
    ) -> UiInstanceId {
        with_world(|world| {
            let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
            let Ok((mut host, mut commands)) = state.get_mut(world) else {
                panic!("ui mount_block_panel params unavailable");
            };
            let id = host.mount_block_panel(&mut commands, root, panel, pos);
            state.apply(world);
            id
        })
    }

    pub fn open_confirm_then(
        self,
        props: ConfirmProps,
        on_complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        with_world(|world| {
            let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
            let Ok((mut host, mut commands)) = state.get_mut(world) else {
                panic!("ui open_confirm_then params unavailable");
            };
            let id = host.open_confirm_then(&mut commands, props, on_complete);
            state.apply(world);
            id
        })
    }

    pub fn open_text_prompt_then(
        self,
        props: TextPromptProps,
        on_complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        with_world(|world| {
            let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
            let Ok((mut host, mut commands)) = state.get_mut(world) else {
                panic!("ui open_text_prompt_then params unavailable");
            };
            let id = host.open_text_prompt_then(&mut commands, props, on_complete);
            state.apply(world);
            id
        })
    }
}
