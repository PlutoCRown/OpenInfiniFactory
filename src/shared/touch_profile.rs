//! 触控 / 虚拟遥感启用判定（Android、iOS、Web 手机平板 UA）

use bevy::prelude::*;

use super::platform::StoragePlatform;

/// 是否启用虚拟遥感与触控游玩配置
#[derive(Resource, Clone, Copy, Debug)]
pub struct TouchProfile {
    pub enabled: bool,
}

// 触控设备固定像素 UI 的额外缩放；虚拟遥感使用 VMin，不受此值影响。
pub const TOUCH_UI_SCALE: f32 = 0.6;

impl TouchProfile {
    pub fn detect_with_force(force_touch: bool) -> Self {
        if force_touch {
            return Self { enabled: true };
        }
        let enabled = match StoragePlatform::current() {
            StoragePlatform::Android | StoragePlatform::Ios => true,
            StoragePlatform::Web => web_ua_is_phone_or_tablet(),
            StoragePlatform::Desktop => false,
        };
        Self { enabled }
    }

    /// 把用户设置的 UI 比例换算为当前输入平台的实际比例
    pub fn effective_ui_scale(self, configured_scale: f32) -> f32 {
        configured_scale
            * if self.enabled {
                TOUCH_UI_SCALE
            } else {
                1.0
            }
    }
}

fn web_ua_is_phone_or_tablet() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return false;
        };
        let Ok(ua) = window.navigator().user_agent() else {
            return false;
        };
        let ua = ua.to_ascii_lowercase();
        ua.contains("iphone")
            || ua.contains("ipod")
            || ua.contains("ipad")
            || ua.contains("android")
            || ua.contains("mobile")
            || ua.contains("tablet")
            || ua.contains("kindle")
            || ua.contains("silk/")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}
