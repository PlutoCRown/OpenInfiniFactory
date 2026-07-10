//! 全局 `ui` / `i18n` 访问层。
//!
//! 由 `bind_ui_scope` / `unbind_ui_scope` 在每帧 UI 处理区间绑定 `World`；
//! 业务代码 `use crate::game::ui::access::{i18n, ui}` 后直接 `i18n.t("key")` / `ui.open_*`，勿再作为参数传递。

use std::cell::Cell;
use std::ptr::NonNull;

use bevy::ecs::system::SystemState;
use bevy::ecs::system::NonSendMarker;
use bevy::ecs::world::World;
use bevy::prelude::*;

/// 声明于使用 `i18n` / `ui` 的系统中，保证其在主线程执行（与 `bind_ui_scope` 同线程）。
pub type UiMainThread = NonSendMarker;

use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::core::confirm_dialog::{ConfirmProps, ConfirmResult};
use crate::game::ui::core::host::{UiHost, UiHostCommands, UiInstanceId};
use crate::game::ui::core::runtime::{UiPanelContext, UiRuntime};
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::shared::i18n::{I18n, Language};

thread_local! {
    static UI_WORLD: Cell<Option<NonNull<World>>> = const { Cell::new(None) };
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct I18nRevision(pub u32);

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

fn bump_i18n_revision(world: &mut World) {
    world.resource_mut::<I18nRevision>().0 += 1;
}

/// 全局 i18n 访问点。用法：`i18n.t("button.save")`
#[allow(non_upper_case_globals)]
pub const i18n: I18nAccess = I18nAccess;

#[derive(Clone, Copy, Debug, Default)]
pub struct I18nAccess;

impl I18nAccess {
    pub fn t(self, key: &'static str) -> String {
        with_world_immut(|world| world.resource::<I18n>().text(key))
    }

    pub fn fmt(self, key: &'static str, values: &[(&str, String)]) -> String {
        with_world_immut(|world| world.resource::<I18n>().fmt(key, values))
    }

    pub fn language(self) -> Language {
        with_world_immut(|world| world.resource::<I18n>().language())
    }

    pub fn set_language(self, language: Language) {
        with_world(|world| {
            world.resource_mut::<I18n>().set_language(language);
            bump_i18n_revision(world);
        });
    }
}

/// 全局 UI 访问点。用法：`ui.open_confirm_then(...)`
#[allow(non_upper_case_globals)]
pub const ui: UiAccess = UiAccess;

#[derive(Clone, Copy, Debug, Default)]
pub struct UiAccess;

impl UiAccess {
    #[allow(dead_code)]
    pub fn modal_open(self) -> bool {
        with_world_immut(|world| world.resource::<UiHost>().modal_open())
    }

    #[allow(dead_code)]
    pub fn is_settings_open(self) -> bool {
        with_world_immut(|world| world.resource::<UiRuntime>().is_settings_open())
    }

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
                }
            };
            let mut state = SystemState::<UiHostCommands>::new(world);
            let mut params = state.get_mut(world).unwrap();
            params.mount_settings(commands, root, context, &settings)
        })
    }

    pub fn unmount_panel(self, panel: UiPanelId, commands: &mut Commands) {
        with_ui(|host| host.unmount_panel(panel, commands));
    }

    pub fn open_confirm_then(
        self,
        props: ConfirmProps,
        on_complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        with_ui(|host| host.open_confirm_then(props, on_complete))
    }

    pub fn open_text_prompt_then(
        self,
        props: TextPromptProps,
        on_complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) -> UiInstanceId {
        with_ui(|host| host.open_text_prompt_then(props, on_complete))
    }
}
