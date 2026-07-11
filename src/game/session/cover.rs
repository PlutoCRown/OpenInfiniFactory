#[cfg(not(target_arch = "wasm32"))]
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};

#[cfg(not(target_arch = "wasm32"))]
use crate::game::cameras::GameplayViewImage;
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::platform::saves_directory;
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::save_format::COVER_FILE;

/// 等待封面截图完成后回到主菜单
#[derive(Resource, Default)]
pub struct PendingMainMenuExit(pub bool);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Message)]
pub struct CoverScreenshotComplete;

#[cfg(not(target_arch = "wasm32"))]
pub fn should_capture_cover(save_name: Option<&str>) -> bool {
    save_name.is_some()
}

#[cfg(target_arch = "wasm32")]
pub fn should_capture_cover(_save_name: Option<&str>) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn begin_cover_capture(
    commands: &mut Commands,
    save_name: &str,
    view_image: &GameplayViewImage,
) {
    let path = saves_directory().join(save_name).join(COVER_FILE);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    commands
        .spawn(Screenshot::image(view_image.0.clone()))
        .observe(save_to_disk(path));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_screenshot_saved_for_exit(
    _: On<ScreenshotCaptured>,
    pending: Res<PendingMainMenuExit>,
    mut complete: MessageWriter<CoverScreenshotComplete>,
) {
    if pending.0 {
        complete.write(CoverScreenshotComplete);
    }
}
