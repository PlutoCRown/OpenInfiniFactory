mod game;
mod shared;
mod tools;

#[cfg(target_arch = "wasm32")]
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

use game::GamePlugin;
use shared::platform;

#[bevy_main]
fn main() {
    App::new()
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
