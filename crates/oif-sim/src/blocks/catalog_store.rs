//! 字符串 id 目录骨架：Vec + HashMap，以及全局 RwLock 读写辅助

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

/// 目录条目需暴露稳定字符串 id
pub trait HasStringId {
    fn string_id(&self) -> &str;
}

/// 目录条目类型标签（用于 register 错误信息）
pub trait CatalogEntry: HasStringId {
    const LABEL: &'static str;
}

/// 句柄：由 u16 索引构造/还原
pub trait CatalogId: Copy + Eq + Hash + From<u16> + Into<u16> {}

impl<T: Copy + Eq + Hash + From<u16> + Into<u16>> CatalogId for T {}

/// 通用字符串 id 目录
#[derive(Clone, Debug)]
pub struct StringIdCatalog<Id, Def> {
    defs: Vec<Def>,
    by_string: HashMap<String, Id>,
}

impl<Id, Def> Default for StringIdCatalog<Id, Def> {
    fn default() -> Self {
        Self {
            defs: Vec::new(),
            by_string: HashMap::new(),
        }
    }
}

impl<Id, Def> StringIdCatalog<Id, Def>
where
    Id: CatalogId,
    Def: CatalogEntry,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn get(&self, id: Id) -> Option<&Def> {
        self.defs.get(Into::<u16>::into(id) as usize)
    }

    pub fn id_by_string(&self, string_id: &str) -> Option<Id> {
        self.by_string.get(string_id).copied()
    }

    pub fn string_id(&self, id: Id) -> Option<&str> {
        self.get(id).map(|d| d.string_id())
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &Def)> {
        self.defs
            .iter()
            .enumerate()
            .map(|(i, def)| (Id::from(i as u16), def))
    }

    /// 注册条目；`string_id` 重复或目录满则返回 Err
    pub fn register(&mut self, def: Def) -> Result<Id, String> {
        let sid = def.string_id();
        if self.by_string.contains_key(sid) {
            return Err(format!("duplicate {} id '{sid}'", Def::LABEL));
        }
        if self.defs.len() >= u16::MAX as usize {
            return Err(format!("{} catalog full", Def::LABEL));
        }
        let id = Id::from(self.defs.len() as u16);
        self.by_string.insert(sid.to_string(), id);
        self.defs.push(def);
        Ok(id)
    }
}

/// 把字符串泄漏为 `'static`（资源包 id / i18n key）
pub fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// 写入全局目录
pub fn install_global<C>(lock: &RwLock<C>, catalog: C) {
    *lock.write().expect("catalog lock") = catalog;
}

/// 克隆全局目录快照
pub fn clone_global<C: Clone>(lock: &RwLock<C>) -> C {
    lock.read().expect("catalog lock").clone()
}

/// 在读锁下查询全局目录
pub fn with_global<C, R>(lock: &RwLock<C>, f: impl FnOnce(&C) -> R) -> R {
    f(&lock.read().expect("catalog lock"))
}

/// 在写锁下修改全局目录
pub fn with_global_mut<C, R>(lock: &RwLock<C>, f: impl FnOnce(&mut C) -> R) -> R {
    f(&mut lock.write().expect("catalog lock"))
}
