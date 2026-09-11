//! UI 只读快照与延迟挂载请求。

use bevy::ecs::system::{NonSendMarker, SystemParam, SystemState};
use bevy::prelude::*;
use std::cell::RefCell;

use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::components::UiIconAssets;
use crate::game::ui::core::confirm_dialog::{ConfirmProps, ConfirmResult};
use crate::game::ui::core::host::{PanelMount, UiHostCommands, UiInstanceId};
use crate::game::ui::core::runtime::UiPanelContext;
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::game::ui::features::save_settings::SaveSettingsUiState;
use crate::game::ui::systems::UiFont;
use crate::shared::i18n::{I18n, Language};
use crate::shared::save::read_save_settings;

/// UI 只读依赖：由 Bevy 声明借用，不包含按钮业务所需的可变资源。
#[derive(SystemParam)]
pub struct UiContext<'w> {
    locale: Res<'w, I18n>,
    icons: Option<Res<'w, UiIconAssets>>,
    font: Option<Res<'w, UiFont>>,
    thread: NonSendMarker,
}

impl UiContext<'_> {
    /// 文案资源是否在本系统上次执行后被替换。
    pub fn locale_changed(&self) -> bool {
        self.locale.is_changed()
    }

    /// 在当前系统调用期间提供类似 React Context 的只读外观数据。
    pub fn enter(&self) -> UiRenderScope {
        let _ = &self.thread;
        UiReadSnapshot {
            i18n: self.locale.as_ref().clone(),
            icons: self.icons.as_deref().cloned(),
            font: self.font.as_ref().map(|font| font.0.clone()),
        }
        .enter()
    }
}

/// 仅在调用栈内持有不可变文案和资产句柄，不持有 World 或业务状态。
#[derive(Clone)]
struct UiReadSnapshot {
    i18n: I18n,
    icons: Option<UiIconAssets>,
    font: Option<Handle<Font>>,
}

impl UiReadSnapshot {
    /// 嵌套挂载结束时恢复外层上下文，各 App 和线程之间不遗留状态。
    fn enter(self) -> UiRenderScope {
        UI_CONTEXT
            .with(|context| UiRenderScope(context.replace(Some(self)), std::marker::PhantomData))
    }
}

/// 作用域退出即恢复原上下文；不允许携带到另一线程。
pub struct UiRenderScope(
    Option<UiReadSnapshot>,
    std::marker::PhantomData<std::rc::Rc<()>>,
);

impl Drop for UiRenderScope {
    fn drop(&mut self) {
        UI_CONTEXT.with(|context| {
            context.replace(self.0.take());
        });
    }
}

thread_local! {
    /// 当前渲染调用栈的只读上下文，绝不保存 World 引用或指针。
    static UI_CONTEXT: RefCell<Option<UiReadSnapshot>> = const { RefCell::new(None) };
}

/// 独占 World 入口显式建立 UI 上下文，返回值不借用 World。
pub fn enter_ui_world(world: &World) -> UiRenderScope {
    UiReadSnapshot {
        i18n: world.resource::<I18n>().clone(),
        icons: world.get_resource::<UiIconAssets>().cloned(),
        font: world.get_resource::<UiFont>().map(|font| font.0.clone()),
    }
    .enter()
}

/// 挂载命令只在 Bevy 应用 Commands 时获取所需资源。
enum UiRequest {
    MountSettings {
        root: Option<Entity>,
        context: UiPanelContext,
    },
    MountSaveSettings {
        root: Option<Entity>,
    },
    MountBlockPanel {
        root: Option<Entity>,
        panel: UiPanelId,
        pos: IVec3,
    },
    UnmountPanel(UiPanelId),
    Confirm {
        props: ConfirmProps,
        complete: Box<dyn FnOnce(ConfirmResult, &mut World) + Send>,
    },
    TextPrompt {
        props: TextPromptProps,
        complete: Box<dyn FnOnce(TextPromptResult, &mut World) + Send>,
    },
}

/// 返回通用 UI 图标快照。
pub fn ui_icons() -> UiIconAssets {
    UI_CONTEXT.with(|context| {
        context
            .borrow()
            .as_ref()
            .and_then(|view| view.icons.clone())
            .expect("UI icons require a render scope")
    })
}

/// 返回当前渲染作用域的字体句柄。
pub fn ui_font() -> Handle<Font> {
    UI_CONTEXT.with(|context| {
        context
            .borrow()
            .as_ref()
            .and_then(|view| view.font.clone())
            .expect("UI font requires a render scope")
    })
}

/// 处理普通 UI 发出的挂载与模态请求。
impl Command for UiRequest {
    type Out = ();
    fn apply(self, world: &mut World) {
        let _scope = enter_ui_world(world);
        match self {
            UiRequest::MountSettings { root, context } => {
                let settings = {
                    let settings = world.resource::<GameSettings>();
                    GameSettings {
                        fov_degrees: settings.fov_degrees,
                        ui_scale: settings.ui_scale,
                        gravity_scale: settings.gravity_scale,
                        mouse_sensitivity_x: settings.mouse_sensitivity_x,
                        mouse_sensitivity_y: settings.mouse_sensitivity_y,
                        virtual_controls_opacity: settings.virtual_controls_opacity,
                        master_volume: settings.master_volume,
                        music_volume: settings.music_volume,
                        sfx_volume: settings.sfx_volume,
                    }
                };
                let touch_enabled = world
                    .resource::<crate::shared::touch_profile::TouchProfile>()
                    .enabled;
                let (panel_w, panel_h) =
                    panel_size(world, crate::game::ui::screens::settings_panel_size);
                let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
                let (mut host, mut commands) = state.get_mut(world).expect("settings host params");
                host.mount_panel(
                    &mut commands,
                    root,
                    PanelMount {
                        panel: UiPanelId::Settings,
                        instance: None,
                        replaces: |panel| panel.is_settings(),
                    },
                    |navigation| navigation.open(UiPanelId::Settings, context),
                    |parent| {
                        crate::game::ui::screens::spawn_settings_panel(
                            parent,
                            &settings,
                            panel_w,
                            panel_h,
                            touch_enabled,
                        )
                    },
                );
                state.apply(world);
            }
            UiRequest::MountSaveSettings { root } => mount_save_settings(world, root),
            UiRequest::MountBlockPanel { root, panel, pos } => {
                let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
                let (mut host, mut commands) =
                    state.get_mut(world).expect("block panel host params");
                host.mount_panel(
                    &mut commands,
                    root,
                    PanelMount {
                        panel,
                        instance: None,
                        replaces: |panel| !panel.is_settings(),
                    },
                    |navigation| navigation.open_block(panel, pos),
                    |parent| {
                        if let Some(hooks) =
                            crate::game::blocks::panels::find_block_panel_hooks(panel)
                        {
                            (hooks.spawn_panel)(parent);
                            (hooks.spawn_overlays)(parent);
                        }
                    },
                );
                state.apply(world);
            }
            UiRequest::UnmountPanel(panel) => {
                let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
                let (mut host, mut commands) = state.get_mut(world).expect("panel host params");
                host.unmount_panel(panel, &mut commands);
                state.apply(world);
            }
            UiRequest::Confirm { props, complete } => {
                let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
                let (mut host, mut commands) = state.get_mut(world).expect("confirm host params");
                host.open_confirm_then(&mut commands, props, complete);
                state.apply(world);
            }
            UiRequest::TextPrompt { props, complete } => {
                let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
                let (mut host, mut commands) = state.get_mut(world).expect("prompt host params");
                host.open_text_prompt_then(&mut commands, props, complete);
                state.apply(world);
            }
        }
    }
}

fn panel_size(world: &mut World, resolve: fn(f32, f32, f32) -> (f32, f32)) -> (f32, f32) {
    use bevy::window::PrimaryWindow;
    let scale = world.resource::<UiScale>().0.max(0.01);
    let (width, height) = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .iter(world)
        .next()
        .map(|window| (window.width(), window.height()))
        .unwrap_or((1280.0, 720.0));
    resolve(width, height, scale)
}

fn mount_save_settings(world: &mut World, root: Option<Entity>) {
    let Some(slot) = world
        .resource::<crate::shared::save::SaveState>()
        .current
        .clone()
    else {
        return;
    };
    let data = read_save_settings(&slot).unwrap_or_default();
    let skybox_bytes = crate::shared::persistent_storage::read_save_bytes(
        &slot.storage_path(),
        crate::shared::save_format::SKYBOX_FILE,
    );
    let edit_mode = world.resource::<crate::game::state::SolutionState>().entry
        == crate::game::state::WorldEntryMode::EditPuzzle;
    let (panel_w, panel_h) = panel_size(world, crate::game::ui::screens::save_settings_panel_size);
    {
        let mut state = world.resource_mut::<SaveSettingsUiState>();
        state.slot = Some(slot.clone());
        state.data = data.clone();
        state.skybox_bytes = skybox_bytes.clone().map(Into::into);
        state.edit_mode = edit_mode;
        state.picker_open = false;
    }
    let view = crate::game::ui::screens::SaveSettingsSpawnCtx {
        data,
        edit_mode,
        panel_w,
        panel_h,
    };
    let mut state = SystemState::<(UiHostCommands, Commands)>::new(world);
    let (mut host, mut commands) = state.get_mut(world).expect("save settings host params");
    host.mount_panel(
        &mut commands,
        root,
        PanelMount {
            panel: UiPanelId::Settings,
            instance: Some(UiInstanceId::SAVE_SETTINGS),
            replaces: |panel| panel.is_settings(),
        },
        |navigation| navigation.open(UiPanelId::Settings, UiPanelContext::SaveSettingsFromPause),
        |parent| crate::game::ui::screens::spawn_save_settings_panel(parent, &view),
    );
    state.apply(world);
}

/// 全局只读文案快照访问点。
#[allow(non_upper_case_globals)]
pub const i18n: I18nAccess = I18nAccess;

#[derive(Clone, Copy, Debug, Default)]
pub struct I18nAccess;

impl I18nAccess {
    /// 使用当前系统声明的文案资源。
    pub fn t(self, key: &'static str) -> String {
        UI_CONTEXT.with(|context| {
            context
                .borrow()
                .as_ref()
                .expect("UI text requires a render scope")
                .i18n
                .t(key)
                .to_owned()
        })
    }
    /// 格式化当前语言的文案。
    pub fn fmt(self, key: &'static str, values: &[(&str, &str)]) -> String {
        UI_CONTEXT.with(|context| {
            context
                .borrow()
                .as_ref()
                .expect("UI text requires a render scope")
                .i18n
                .fmt(key, values)
        })
    }
    /// 当前调用栈正在渲染的语言。
    pub fn language(self) -> Language {
        UI_CONTEXT.with(|context| {
            context
                .borrow()
                .as_ref()
                .expect("UI text requires a render scope")
                .i18n
                .language()
        })
    }
}

/// UI 挂载与模态请求入口。
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
    ) {
        commands.queue(UiRequest::MountSettings { root, context });
    }
    pub fn unmount_panel(self, panel: UiPanelId, commands: &mut Commands) {
        commands.queue(UiRequest::UnmountPanel(panel));
    }
    pub fn mount_save_settings(self, commands: &mut Commands, root: Option<Entity>) {
        commands.queue(UiRequest::MountSaveSettings { root });
    }
    pub fn mount_block_panel(
        self,
        commands: &mut Commands,
        root: Option<Entity>,
        panel: UiPanelId,
        pos: IVec3,
    ) {
        commands.queue(UiRequest::MountBlockPanel { root, panel, pos });
    }
    pub fn open_confirm_then(
        self,
        commands: &mut Commands,
        props: ConfirmProps,
        complete: impl FnOnce(ConfirmResult, &mut World) + Send + 'static,
    ) {
        commands.queue(UiRequest::Confirm {
            props,
            complete: Box::new(move |result, world| {
                let _scope = enter_ui_world(world);
                complete(result, world);
            }),
        });
    }
    pub fn open_text_prompt_then(
        self,
        commands: &mut Commands,
        props: TextPromptProps,
        complete: impl FnOnce(TextPromptResult, &mut World) + Send + 'static,
    ) {
        commands.queue(UiRequest::TextPrompt {
            props,
            complete: Box::new(move |result, world| {
                let _scope = enter_ui_world(world);
                complete(result, world);
            }),
        });
    }
}
