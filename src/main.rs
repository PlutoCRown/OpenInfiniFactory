use open_infinifactory::game::GamePlugin;
use open_infinifactory::shared::launch::LaunchOptions;
use open_infinifactory::shared::platform;

#[cfg(target_arch = "wasm32")]
use bevy::asset::AssetMetaCheck;
use bevy::log::{LogPlugin, DEFAULT_FILTER};
use bevy::prelude::*;

/// Android 入口：android-activity 通过 extern "Rust" 调用此函数
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: bevy::android::android_activity::AndroidApp) {
    let _ = bevy::android::ANDROID_APP.set(android_app);
    main();
}

fn main() {
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
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    // CJK 文本布局会触发 icu_provider 缺分词模型警告（Bevy #24094）
                    filter: format!("{DEFAULT_FILTER},icu_provider=error"),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
