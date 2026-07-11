//! Bevy 桥：启动 hydrate、轮询就绪、异步刷盘。

use std::sync::{Mutex, OnceLock};

use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};

use super::backend;
use super::vault::{is_ready, lock_vault};

/// 存储层已完成首次加载，可读配置/存档列表
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageReady(pub bool);

/// 异步 hydrate 完成时的条目（Web / 可选原生后台加载）
static HYDRATE_RESULT: OnceLock<Mutex<Option<std::collections::HashMap<String, Vec<u8>>>>> =
    OnceLock::new();

/// 注册存储 runtime：Web 异步灌 IndexedDB；原生亦可后台重灌
pub struct StoragePlugin;

impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StorageReady>()
            .init_resource::<PersistFlushState>()
            .add_systems(Startup, begin_storage_hydrate)
            .add_systems(
                Update,
                (poll_storage_hydrate, flush_persist_queue).chain(),
            );
    }
}

#[derive(Resource, Default)]
struct PersistFlushState {
    task: Option<Task<()>>,
}

#[cfg(target_arch = "wasm32")]
fn begin_storage_hydrate() {
    backend::web_idb::begin_hydrate(|entries| {
        *HYDRATE_RESULT
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(entries);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn begin_storage_hydrate(mut ready: ResMut<StorageReady>) {
    drop(lock_vault());
    ready.0 = true;
    bevy::log::info!("storage ready ({:?})", backend::backend_kind());
}

fn poll_storage_hydrate(mut ready: ResMut<StorageReady>) {
    if ready.0 {
        return;
    }
    let Some(slot) = HYDRATE_RESULT.get() else {
        return;
    };
    let mut guard = slot.lock().unwrap_or_else(|p| p.into_inner());
    let Some(entries) = guard.take() else {
        return;
    };
    lock_vault().replace_all(entries);
    ready.0 = true;
    bevy::log::info!("storage ready ({:?})", backend::backend_kind());
}

fn flush_persist_queue(mut state: ResMut<PersistFlushState>) {
    if let Some(task) = state.task.as_mut() {
        if !task.is_finished() {
            return;
        }
        state.task = None;
    }

    if !is_ready() {
        return;
    }
    let ops = lock_vault().drain_persist();
    if ops.is_empty() {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        backend::web_idb::begin_apply_ops(ops);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        state.task = Some(IoTaskPool::get().spawn(async move {
            backend::native_fs::apply_ops(&ops);
        }));
    }
}
