//! 性能优化模块
//!
//! ## 包含特性
//!
//! - LRU 缓存：热点数据缓存
//! - 延迟加载：按需加载分组内容
//! - 索引加速：URL/用户名索引

use crate::vault::entry::{Entry, EntryId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub struct LruCache<K, V> {
    capacity: usize,
    store: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            store: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(v) = self.store.get(key) {
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.store.contains_key(&key) {
            self.store.insert(key.clone(), value);
            self.order.retain(|k| k != &key);
            self.order.push_back(key);
            return;
        }

        if self.store.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.store.remove(&oldest);
            }
        }

        self.store.insert(key.clone(), value);
        self.order.push_back(key);
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = (K, V)>) {
        for (k, v) in items {
            self.put(k, v);
        }
    }

    pub fn clear(&mut self) {
        self.store.clear();
        self.order.clear();
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

pub struct EntryCache {
    by_id: LruCache<EntryId, Arc<Entry>>,
    by_url: LruCache<String, Vec<EntryId>>,
    by_username: LruCache<String, Vec<EntryId>>,
    #[allow(dead_code)]
    capacity: usize,
}

impl EntryCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            by_id: LruCache::new(capacity),
            by_url: LruCache::new(capacity / 10),
            by_username: LruCache::new(capacity / 10),
            capacity,
        }
    }

    pub fn get(&mut self, id: &EntryId) -> Option<Arc<Entry>> {
        self.by_id.get(id)
    }

    pub fn put(&mut self, entry: Arc<Entry>) {
        self.by_id.put(entry.id.clone(), entry);
    }

    pub fn put_all(&mut self, entries: impl IntoIterator<Item = Arc<Entry>>) {
        for entry in entries {
            self.put(entry);
        }
    }

    pub fn get_by_url(&mut self, url: &str) -> Option<Vec<EntryId>> {
        self.by_url.get(&url.to_lowercase())
    }

    pub fn index_url(&mut self, url: &str, ids: Vec<EntryId>) {
        self.by_url.put(url.to_lowercase(), ids);
    }

    pub fn get_by_username(&mut self, username: &str) -> Option<Vec<EntryId>> {
        self.by_username.get(&username.to_lowercase())
    }

    pub fn index_username(&mut self, username: &str, ids: Vec<EntryId>) {
        self.by_username.put(username.to_lowercase(), ids);
    }

    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_url.clear();
        self.by_username.clear();
    }

    pub fn rebuild_index<I: Iterator<Item = Arc<Entry>>>(&mut self, entries: I) {
        self.clear();

        for entry in entries {
            if let Some(ref url) = entry.url {
                let id = entry.id.clone();
                let mut ids = self.by_url.get(&url.to_lowercase()).unwrap_or_default();
                ids.push(id);
                self.by_url.put(url.to_lowercase(), ids);
            }

            if !entry.username.is_empty() {
                let id = entry.id.clone();
                let mut ids = self.by_username.get(&entry.username.to_lowercase()).unwrap_or_default();
                ids.push(id);
                self.by_username.put(entry.username.to_lowercase(), ids);
            }

            self.put(entry);
        }
    }
}

pub struct LazyGroupContent {
    loaded: bool,
    entry_ids: Vec<EntryId>,
}

impl LazyGroupContent {
    pub fn new(entry_ids: Vec<EntryId>) -> Self {
        Self {
            loaded: false,
            entry_ids,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn mark_loaded(&mut self) {
        self.loaded = true;
    }

    pub fn entry_ids(&self) -> &[EntryId] {
        &self.entry_ids
    }

    pub fn set_entry_ids(&mut self, ids: Vec<EntryId>) {
        self.entry_ids = ids;
        self.loaded = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProgress {
    pub total: usize,
    pub loaded: usize,
    pub current: String,
}

impl LoadProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            loaded: 0,
            current: String::new(),
        }
    }

    pub fn set_current(&mut self, current: impl Into<String>) {
        self.current = current.into();
    }

    pub fn increment(&mut self) {
        self.loaded += 1;
    }

    pub fn finish(&mut self) {
        self.loaded = self.total;
        self.current = String::new();
    }

    pub fn progress(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            self.loaded as f32 / self.total as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(3);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));

        cache.put("d", 4);

        assert!(cache.get(&"b").is_none());
        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"c"), Some(3));
        assert_eq!(cache.get(&"d"), Some(4));
    }

    #[test]
    fn test_load_progress() {
        let mut progress = LoadProgress::new(100);
        assert_eq!(progress.progress(), 0.0);

        progress.increment();
        progress.set_current("Loading entries");
        assert_eq!(progress.progress(), 0.01);

        progress.finish();
        assert_eq!(progress.progress(), 1.0);
    }
}