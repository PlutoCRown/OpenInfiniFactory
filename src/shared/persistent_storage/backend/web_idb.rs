//! Web IndexedDB 后端（异步 hydrate / 落盘；兼容从 localStorage 迁移）。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use js_sys::Uint8Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{IdbDatabase, IdbObjectStore, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

use crate::shared::persistent_storage::vault::PersistOp;

const DB_NAME: &str = "open_infinifactory";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "kv";
const LEGACY_LS_PREFIX: &str = "open_infinifactory:";

/// 打开 DB、迁移旧 localStorage，并把全部 KV 回调给调用方
pub fn begin_hydrate(on_done: impl FnOnce(HashMap<String, Vec<u8>>) + 'static) {
    let on_done = Rc::new(RefCell::new(Some(on_done)));
    open_db({
        let on_done = Rc::clone(&on_done);
        move |db| match db {
            None => {
                if let Some(cb) = on_done.borrow_mut().take() {
                    cb(HashMap::new());
                }
            }
            Some(db) => {
                migrate_local_storage(&db, {
                    let on_done = Rc::clone(&on_done);
                    move |db| {
                        read_all(&db, move |entries| {
                            if let Some(cb) = on_done.borrow_mut().take() {
                                cb(entries);
                            }
                        });
                    }
                });
            }
        }
    });
}

/// 把一批变更写入 IndexedDB（fire-and-forget）
pub fn begin_apply_ops(ops: Vec<PersistOp>) {
    if ops.is_empty() {
        return;
    }
    open_db(move |db| {
        let Some(db) = db else {
            return;
        };
        let Ok(tx) = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
        else {
            bevy::log::warn!("IndexedDB: open write transaction failed");
            return;
        };
        let Ok(store) = tx.object_store(STORE_NAME) else {
            return;
        };
        for op in ops {
            match op {
                PersistOp::Put { key, value } => {
                    let bytes = Uint8Array::new_with_length(value.len() as u32);
                    bytes.copy_from(&value);
                    let _ = store.put_with_key(&bytes, &JsValue::from_str(&key));
                }
                PersistOp::RemovePrefix { prefix } => {
                    delete_prefix(&store, &prefix);
                }
            }
        }
    });
}

fn open_db(on_open: impl FnOnce(Option<IdbDatabase>) + 'static) {
    let on_open = Rc::new(RefCell::new(Some(on_open)));
    let Some(window) = web_sys::window() else {
        if let Some(cb) = on_open.borrow_mut().take() {
            cb(None);
        }
        return;
    };
    let Ok(Some(factory)) = window.indexed_db() else {
        bevy::log::warn!("IndexedDB unavailable");
        if let Some(cb) = on_open.borrow_mut().take() {
            cb(None);
        }
        return;
    };
    let Ok(request) = factory.open_with_u32(DB_NAME, DB_VERSION) else {
        if let Some(cb) = on_open.borrow_mut().take() {
            cb(None);
        }
        return;
    };

    let upgrade = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(target) = event.target() else {
            return;
        };
        let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() else {
            return;
        };
        let Ok(db) = req
            .result()
            .map_err(|_| ())
            .and_then(|v| v.dyn_into::<IdbDatabase>().map_err(|_| ()))
        else {
            return;
        };
        if db.object_store_names().contains(STORE_NAME) {
            return;
        }
        let _ = db.create_object_store(STORE_NAME);
    }) as Box<dyn FnMut(_)>);
    request.set_onupgradeneeded(Some(upgrade.as_ref().unchecked_ref()));
    upgrade.forget();

    let success_open = Rc::clone(&on_open);
    let success = Closure::once(Box::new(move |event: web_sys::Event| {
        let db = event
            .target()
            .and_then(|t| t.dyn_into::<IdbOpenDbRequest>().ok())
            .and_then(|req| req.result().ok())
            .and_then(|v| v.dyn_into::<IdbDatabase>().ok());
        if let Some(cb) = success_open.borrow_mut().take() {
            cb(db);
        }
    }) as Box<dyn FnMut(_)>);
    request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
    success.forget();

    let error_cb = Rc::clone(&on_open);
    let error = Closure::once(Box::new(move |_event: web_sys::Event| {
        bevy::log::warn!("IndexedDB open failed");
        if let Some(cb) = error_cb.borrow_mut().take() {
            cb(None);
        }
    }) as Box<dyn FnMut(_)>);
    request.set_onerror(Some(error.as_ref().unchecked_ref()));
    error.forget();
}

fn read_all(db: &IdbDatabase, on_done: impl FnOnce(HashMap<String, Vec<u8>>) + 'static) {
    let on_done = Rc::new(RefCell::new(Some(on_done)));
    let Ok(tx) = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readonly) else {
        if let Some(cb) = on_done.borrow_mut().take() {
            cb(HashMap::new());
        }
        return;
    };
    let Ok(store) = tx.object_store(STORE_NAME) else {
        if let Some(cb) = on_done.borrow_mut().take() {
            cb(HashMap::new());
        }
        return;
    };
    let Ok(request) = store.open_cursor() else {
        if let Some(cb) = on_done.borrow_mut().take() {
            cb(HashMap::new());
        }
        return;
    };

    let entries = Rc::new(RefCell::new(HashMap::new()));
    let entries_cb = Rc::clone(&entries);
    let on_done_cb = Rc::clone(&on_done);
    let success = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(req) = event.target().and_then(|t| t.dyn_into::<IdbRequest>().ok()) else {
            return;
        };
        let Ok(result) = req.result() else {
            return;
        };
        if result.is_null() {
            if let Some(cb) = on_done_cb.borrow_mut().take() {
                cb(entries_cb.borrow().clone());
            }
            return;
        }
        let Ok(cursor) = result.dyn_into::<web_sys::IdbCursorWithValue>() else {
            return;
        };
        let key = cursor.key().ok().and_then(|k| k.as_string());
        let value = cursor.value().ok().and_then(js_value_to_bytes);
        if let (Some(key), Some(value)) = (key, value) {
            entries_cb.borrow_mut().insert(key, value);
        }
        let _ = cursor.continue_();
    }) as Box<dyn FnMut(_)>);
    request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
    success.forget();

    let err_done = Rc::clone(&on_done);
    let error = Closure::once(Box::new(move |_event: web_sys::Event| {
        bevy::log::warn!("IndexedDB cursor failed");
        if let Some(cb) = err_done.borrow_mut().take() {
            cb(HashMap::new());
        }
    }) as Box<dyn FnMut(_)>);
    request.set_onerror(Some(error.as_ref().unchecked_ref()));
    error.forget();
}

fn delete_prefix(store: &IdbObjectStore, prefix: &str) {
    let Ok(request) = store.open_cursor() else {
        return;
    };
    let prefix = prefix.to_string();
    let success = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(req) = event.target().and_then(|t| t.dyn_into::<IdbRequest>().ok()) else {
            return;
        };
        let Ok(result) = req.result() else {
            return;
        };
        if result.is_null() {
            return;
        }
        let Ok(cursor) = result.dyn_into::<web_sys::IdbCursorWithValue>() else {
            return;
        };
        if let Some(key) = cursor.key().ok().and_then(|k| k.as_string()) {
            if key.starts_with(&prefix) {
                let _ = cursor.delete();
            }
        }
        let _ = cursor.continue_();
    }) as Box<dyn FnMut(_)>);
    request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
    success.forget();
}

fn migrate_local_storage(db: &IdbDatabase, then: impl FnOnce(IdbDatabase) + 'static) {
    let Some(window) = web_sys::window() else {
        then(db.clone());
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        then(db.clone());
        return;
    };
    let Ok(length) = storage.length() else {
        then(db.clone());
        return;
    };

    let mut migrated = Vec::new();
    for index in 0..length {
        let Ok(Some(key)) = storage.key(index) else {
            continue;
        };
        let Some(relative) = key.strip_prefix(LEGACY_LS_PREFIX) else {
            continue;
        };
        let Ok(Some(value)) = storage.get_item(&key) else {
            continue;
        };
        migrated.push((relative.to_string(), value, key));
    }
    if migrated.is_empty() {
        then(db.clone());
        return;
    }

    let Ok(tx) = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite) else {
        then(db.clone());
        return;
    };
    let Ok(store) = tx.object_store(STORE_NAME) else {
        then(db.clone());
        return;
    };

    for (relative, value, ls_key) in &migrated {
        let bytes = decode_legacy_value(relative, value);
        let uint8 = Uint8Array::new_with_length(bytes.len() as u32);
        uint8.copy_from(&bytes);
        let _ = store.put_with_key(&uint8, &JsValue::from_str(relative));
        let _ = storage.remove_item(ls_key);
    }
    bevy::log::info!(
        "Migrated {} localStorage entries into IndexedDB",
        migrated.len()
    );
    then(db.clone());
}

fn decode_legacy_value(relative: &str, value: &str) -> Vec<u8> {
    if relative.ends_with(".bin") || relative.ends_with(".png") {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD
            .decode(value)
            .unwrap_or_else(|_| value.as_bytes().to_vec())
    } else {
        value.as_bytes().to_vec()
    }
}

fn js_value_to_bytes(value: JsValue) -> Option<Vec<u8>> {
    if let Ok(array) = value.clone().dyn_into::<Uint8Array>() {
        let mut bytes = vec![0u8; array.length() as usize];
        array.copy_to(&mut bytes);
        return Some(bytes);
    }
    if let Ok(buffer) = value.dyn_into::<js_sys::ArrayBuffer>() {
        let array = Uint8Array::new(&buffer);
        let mut bytes = vec![0u8; array.length() as usize];
        array.copy_to(&mut bytes);
        return Some(bytes);
    }
    None
}
