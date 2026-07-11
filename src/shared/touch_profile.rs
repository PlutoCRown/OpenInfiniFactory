//! 触控 / 虚拟遥感启用判定（Android、iOS、Web 手机平板 UA）

use bevy::prelude::*;

use super::platform::StoragePlatform;

/// 是否启用虚拟遥感与触控游玩配置
#[derive(Resource, Clone, Copy, Debug)]
pub struct TouchProfile {
    pub enabled: bool,
}

impl TouchProfile {
    pub fn detect() -> Self {
        Self::detect_with_force(false)
    }

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
