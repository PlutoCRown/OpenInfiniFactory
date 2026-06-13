use open_infinifactory::game::GamePlugin;
use open_infinifactory::shared::launch::LaunchOptions;
use open_infinifactory::shared::platform;

#[cfg(target_arch = "wasm32")]
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

#[bevy_main]
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
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
