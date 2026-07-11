//! 平台存储后端：桌面/移动文件系统，Web IndexedDB。

#[cfg(not(target_arch = "wasm32"))]
pub mod native_fs;

#[cfg(target_arch = "wasm32")]
pub mod web_idb;

use crate::shared::platform::StoragePlatform;

/// 当前编译目标应使用的持久化后端种类
pub fn backend_kind() -> StorageBackendKind {
    match StoragePlatform::current() {
        StoragePlatform::Web => StorageBackendKind::IndexedDb,
        StoragePlatform::Desktop | StoragePlatform::Android | StoragePlatform::Ios => {
            StorageBackendKind::FileSystem
        }
    }
}

/// 持久化后端种类（便于日志与后续扩展）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBackendKind {
    FileSystem,
    IndexedDb,
}
