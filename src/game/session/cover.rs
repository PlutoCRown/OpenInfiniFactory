use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::IoTaskPool;

#[cfg(not(target_arch = "wasm32"))]
use crate::game::cameras::GameplayViewImage;
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::persistent_storage;
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::save::SaveSlot;
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::save_format::COVER_FILE;

/// 延后一帧再执行的退出请求（先画出「保存中」）
#[derive(Clone, Copy, Debug)]
pub struct DeferredMainMenuExit {
    pub save_first: bool,
    pub invalidate_solutions: bool,
    /// 剩余等待帧；收到请求时置 2，减到 0 后再真正存档/退出
    pub hold_frames: u8,
}

/// 退出主菜单的排队与封面截图等待
#[derive(Resource, Default)]
pub struct PendingMainMenuExit {
    pub waiting_cover: bool,
    pub deferred: Option<DeferredMainMenuExit>,
    /// 回主菜单的 OnExit 拆景完成后再清 SessionBusy
    pub release_busy_after_menu: bool,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Message)]
pub struct CoverScreenshotComplete;

#[cfg(not(target_arch = "wasm32"))]
/// 有当前存档槽时才写封面（调用方须保证本次已经写过档）
pub fn should_capture_cover(save_slot: Option<&SaveSlot>) -> bool {
    save_slot.is_some()
}

#[cfg(target_arch = "wasm32")]
pub fn should_capture_cover(_save_slot: Option<&crate::shared::save::SaveSlot>) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
/// 截取游玩画面为封面，编码与落盘放到 IO 线程，避免主线程卡死
pub fn begin_cover_capture(
    commands: &mut Commands,
    slot: &crate::shared::save::SaveSlot,
    view_image: &GameplayViewImage,
) {
    let storage_path = slot.storage_path();
    commands
        .spawn(Screenshot::image(view_image.0.clone()))
        .observe(move |captured: On<ScreenshotCaptured>| {
            let image = captured.image.clone();
            let storage_path = storage_path.clone();
            IoTaskPool::get().spawn(async move {
                let Ok(dyn_img) = image.try_into_dynamic() else {
                    bevy::log::warn!("cover capture: failed to convert image");
                    return;
                };
                // 与 Bevy save_to_disk 一致：丢掉 alpha / HDR 亮度通道
                let rgb = dyn_img.to_rgb8();
                let mut bytes = std::io::Cursor::new(Vec::new());
                if let Err(error) = rgb.write_to(&mut bytes, image::ImageFormat::Png) {
                    bevy::log::warn!("cover capture: png encode failed: {error}");
                    return;
                }
                if !persistent_storage::write_save_bytes(
                    &storage_path,
                    COVER_FILE,
                    &bytes.into_inner(),
                ) {
                    bevy::log::warn!("cover capture: vault write failed for `{storage_path}`");
                }
            });
        });
}

#[cfg(not(target_arch = "wasm32"))]
/// 封面截到后通知「保存并退出」流程
pub fn on_screenshot_saved_for_exit(
    _: On<ScreenshotCaptured>,
    pending: Res<PendingMainMenuExit>,
    mut complete: MessageWriter<CoverScreenshotComplete>,
) {
    if pending.waiting_cover {
        complete.write(CoverScreenshotComplete);
    }
}
