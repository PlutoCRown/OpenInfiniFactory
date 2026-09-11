use bevy::prelude::*;

use crate::game::state::UiPanelId;

/// 主菜单当前页面。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StartMenuPage {
    #[default]
    Main,
    SaveList,
}

/// 游玩期间可叠加在 HUD 上的标准覆盖层。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiOverlay {
    Inventory,
    Pause,
    Tutorial,
}

/// 覆盖层关闭结果；功能插件各自处理关闭后的业务。
#[derive(Event)]
pub struct OverlayClosed(pub UiOverlay);

/// 当前唯一允许打开的模态框种类。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiModal {
    Confirm,
    TextPrompt,
}

/// 面板打开时携带的业务上下文。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPanelContext {
    SettingsFromStartMenu,
    SettingsFromPause,
    SaveSettingsFromPause,
    Block { pos: IVec3 },
}

/// 导航栈中的面板会话。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelSession {
    pub panel: UiPanelId,
    pub context: UiPanelContext,
}

/// 教程面板当前展示到的位置。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialSession {
    pub tutorial: String,
    pub step: usize,
}

/// UI 宿主持有的实体挂载登记；页面意图仅由 UiNavigation 决定。
#[derive(Resource, Default)]
pub struct UiMountState {
    pub main_menu: Option<Entity>,
    pub save_list: Option<Entity>,
    pub session_busy: Option<Entity>,
    pub inventory: Option<Entity>,
    pub pause: Option<Entity>,
    pub tutorial: Option<Entity>,
}

/// UI 唯一权威导航状态；实体挂载只是它的渲染结果。
#[derive(Resource, Debug, Default)]
pub struct UiNavigation {
    start_menu: StartMenuPage,
    overlays: Vec<UiOverlay>,
    panels: Vec<UiPanelSession>,
    modal: Option<UiModal>,
    tutorial: Option<TutorialSession>,
}

impl UiNavigation {
    pub fn reset_for_playing(&mut self) {
        self.overlays.clear();
        self.panels.clear();
        self.modal = None;
        self.tutorial = None;
        self.start_menu = StartMenuPage::Main;
    }

    pub fn reset_for_start_menu(&mut self) {
        self.reset_for_playing();
    }

    pub fn start_menu(&self) -> StartMenuPage {
        self.start_menu
    }

    pub fn show_start_menu(&mut self, page: StartMenuPage) {
        self.start_menu = page;
    }

    pub fn is_paused(&self) -> bool {
        self.overlays.contains(&UiOverlay::Pause)
    }

    pub fn is_inventory_open(&self) -> bool {
        self.overlays.contains(&UiOverlay::Inventory)
    }

    pub fn open_pause(&mut self) {
        self.close_inventory();
        self.push_overlay(UiOverlay::Pause);
    }

    pub fn close_pause(&mut self) {
        self.close_overlay(UiOverlay::Pause);
    }

    pub fn toggle_pause(&mut self) -> bool {
        if self.is_paused() {
            self.close_pause();
            false
        } else {
            self.open_pause();
            true
        }
    }

    pub fn open_inventory(&mut self) {
        self.close_pause();
        self.push_overlay(UiOverlay::Inventory);
    }

    pub fn close_inventory(&mut self) {
        self.close_overlay(UiOverlay::Inventory);
    }

    pub fn active_play(&self) -> bool {
        self.overlays.is_empty() && self.panels.is_empty() && self.modal.is_none()
    }

    pub fn open(&mut self, panel: UiPanelId, context: UiPanelContext) {
        self.panels.retain(|session| session.panel != panel);
        self.panels.push(UiPanelSession { panel, context });
    }

    pub fn open_block(&mut self, panel: UiPanelId, pos: IVec3) {
        self.panels.retain(|session| session.panel.is_settings());
        self.open(panel, UiPanelContext::Block { pos });
    }

    pub fn close_panel(&mut self, panel: UiPanelId) {
        self.panels.retain(|session| session.panel != panel);
    }

    pub fn close_all_panels(&mut self) {
        self.panels.clear();
    }

    pub fn active(&self) -> Option<UiPanelSession> {
        self.panels.last().copied()
    }

    pub fn active_panel(&self) -> Option<UiPanelId> {
        self.active().map(|session| session.panel)
    }

    pub fn is_settings_open(&self) -> bool {
        self.active().is_some_and(|session| {
            session.panel.is_settings()
                && matches!(
                    session.context,
                    UiPanelContext::SettingsFromStartMenu | UiPanelContext::SettingsFromPause
                )
        })
    }

    pub fn is_save_settings_open(&self) -> bool {
        self.active().is_some_and(|session| {
            session.panel.is_settings() && session.context == UiPanelContext::SaveSettingsFromPause
        })
    }

    pub fn blocks_gameplay(&self) -> bool {
        !self.active_play()
    }

    pub fn active_block_pos(&self) -> Option<IVec3> {
        match self.active()?.context {
            UiPanelContext::Block { pos } => Some(pos),
            _ => None,
        }
    }

    pub fn panel_layer(&self, panel: UiPanelId) -> Option<usize> {
        self.panels
            .iter()
            .position(|session| session.panel == panel)
    }

    pub fn top_modal_layer(&self) -> Option<usize> {
        self.panels
            .iter()
            .rposition(|session| session.panel.is_blocking_gameplay())
    }

    pub fn modal(&self) -> Option<UiModal> {
        self.modal
    }

    pub fn open_modal(&mut self, modal: UiModal) {
        self.modal = Some(modal);
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    pub fn tutorial(&self) -> Option<&TutorialSession> {
        self.tutorial.as_ref()
    }

    pub fn open_tutorial(&mut self, tutorial: impl Into<String>) {
        self.close_inventory();
        self.close_pause();
        self.tutorial = Some(TutorialSession {
            tutorial: tutorial.into(),
            step: 0,
        });
        self.push_overlay(UiOverlay::Tutorial);
    }

    pub fn set_tutorial_step(&mut self, step: usize) {
        if let Some(tutorial) = self.tutorial.as_mut() {
            tutorial.step = step;
        }
    }

    pub fn close_tutorial(&mut self) {
        self.tutorial = None;
        self.close_overlay(UiOverlay::Tutorial);
    }

    fn push_overlay(&mut self, overlay: UiOverlay) {
        self.overlays.retain(|current| *current != overlay);
        self.overlays.push(overlay);
    }

    fn close_overlay(&mut self, overlay: UiOverlay) {
        self.overlays.retain(|current| *current != overlay);
    }
}

/// 把面板实体关联到唯一导航状态中的面板类型。
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPanelBinding(pub UiPanelId);
