pub mod game;
pub mod scene;
pub mod shared;
pub mod sim_bridge;

#[cfg(not(target_arch = "wasm32"))]
pub mod debug_http;

/// Android 入口：android-activity 通过 extern "Rust" 调用此函数
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: bevy::android::android_activity::AndroidApp) {
    let _ = bevy::android::ANDROID_APP.set(android_app);
    main_entry();
}

use bevy::log::{LogPlugin, DEFAULT_FILTER};
use bevy::prelude::*;
use game::GamePlugin;
use shared::launch::LaunchOptions;
use shared::platform;

#[cfg(target_arch = "wasm32")]
use bevy::asset::AssetMetaCheck;

/// 游戏启动入口（桌面、Android、WASM 共用）
pub fn main_entry() {
    App::new()
        .insert_resource(LaunchOptions::from_args())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: platform::asset_path(),
                    #[cfg(target_arch = "wasm32")]
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "OpenInfiniFactory Prototype".to_string(),
                        resolution: (1280, 720).into(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        // 玩法默认关闭 IME，避免 Shift 等键被输入法抢走；文本框获焦时再打开
                        ime_enabled: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: format!("{DEFAULT_FILTER},icu_provider=error"),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
