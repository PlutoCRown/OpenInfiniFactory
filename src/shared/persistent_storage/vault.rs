//! 内存镜像：游戏侧同步读写；落盘由 runtime 异步刷出。

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

/// 待写入/删除的持久化任务
#[derive(Clone, Debug)]
pub enum PersistOp {
    Put { key: String, value: Vec<u8> },
    RemovePrefix { prefix: String },
}

/// 进程内 KV 镜像与落盘队列
#[derive(Default)]
pub struct MemoryVault {
    pub entries: HashMap<String, Vec<u8>>,
    pub ready: bool,
    pub persist_queue: VecDeque<PersistOp>,
}

impl MemoryVault {
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).map(|v| v.as_slice())
    }

    pub fn get_text(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) {
        self.entries.insert(key.clone(), value.clone());
        self.persist_queue.push_back(PersistOp::Put { key, value });
    }

    pub fn put_text(&mut self, key: String, value: &str) {
        self.put(key, value.as_bytes().to_vec());
    }

    pub fn remove_prefix(&mut self, prefix: &str) -> bool {
        let keys: Vec<String> = self
            .entries
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect();
        if keys.is_empty() {
            return false;
        }
        for key in &keys {
            self.entries.remove(key);
        }
        self.persist_queue.push_back(PersistOp::RemovePrefix {
            prefix: prefix.to_string(),
        });
        true
    }

    pub fn rename_prefix(&mut self, old_prefix: &str, new_prefix: &str) -> bool {
        let pairs: Vec<(String, Vec<u8>)> = self
            .entries
            .iter()
            .filter_map(|(key, value)| {
                key.strip_prefix(old_prefix)
                    .map(|rest| (format!("{new_prefix}{rest}"), value.clone()))
            })
            .collect();
        if pairs.is_empty() {
            return false;
        }
        if self.entries.keys().any(|key| key.starts_with(new_prefix)) {
            return false;
        }
        self.remove_prefix(old_prefix);
        for (key, value) in pairs {
            self.put(key, value);
        }
        true
    }

    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut keys: Vec<String> = self
            .entries
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    pub fn replace_all(&mut self, entries: HashMap<String, Vec<u8>>) {
        self.entries = entries;
        self.persist_queue.clear();
        self.ready = true;
    }

    pub fn drain_persist(&mut self) -> Vec<PersistOp> {
        self.persist_queue.drain(..).collect()
    }
}

static VAULT: OnceLock<Mutex<MemoryVault>> = OnceLock::new();

/// 取得全局镜像锁；原生端首次访问会同步灌入文件系统
pub fn lock_vault() -> std::sync::MutexGuard<'static, MemoryVault> {
    let mutex = VAULT.get_or_init(|| Mutex::new(MemoryVault::default()));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut guard = mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if !guard.ready {
            let entries = crate::shared::persistent_storage::backend::native_fs::load_all();
            guard.replace_all(entries);
        }
        guard
    }
    #[cfg(target_arch = "wasm32")]
    {
        mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// 存储是否已对游戏可读（Web 需等 IndexedDB hydrate）
pub fn is_ready() -> bool {
    lock_vault().ready
}
