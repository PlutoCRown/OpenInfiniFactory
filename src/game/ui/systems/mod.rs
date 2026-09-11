// UI 字体资源与字体应用系统。
mod font;
// UI 指针悬停状态系统。
mod hover;
// 游戏内 HUD 显示系统。
mod hud;
// UI 通用图标加载系统。
mod icons;
// 本地化文本刷新系统。
mod localized;
// 浮动面板交互与层级系统。
mod panels;
// 游戏状态栏文本系统。
mod status;

pub use font::{UiFont, apply_ui_font, load_ui_font};
pub use hover::{ui_hovered, ui_unhovered};
pub use hud::update_hud_visibility;
pub use icons::load_ui_icons;
pub use localized::update_localized_ui;
pub use panels::{
    PanelCloseDeps, dismiss_dropdowns_on_outside_click, panel_close_clicked, panel_drag_ended,
    panel_drag_started, panel_dragged, update_panel_visibility, update_ui_layers,
};
pub use status::update_status_ui;
