#![allow(unused)]

use core::{
    fmt::Debug,
    hash::{Hash, Hasher},
};
use std::fmt::Formatter;

// Holds reference to a key
#[derive(Debug)]
pub struct KeyRef<K> {
    k: *const K,
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &KeyRef<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

// Used to apply callback for evicted entry
pub trait OnEvictCallback {
    fn on_evict<K, V>(&self, key: &K, val: &V);
}

// Callback for loop eviction
#[derive(Debug, Clone, Copy)]
pub struct DefaultEvictCallback;

impl OnEvictCallback for DefaultEvictCallback {
    fn on_evict<K, V>(&self, key: &K, val: &V) {}
}

// Return when put entry in cache
pub enum PutResult<K, V> {
    // key doesn't exist, and cache has space to make new entry
    Put,
    // key already exists, update the value
    Update(V),
    // key doesn't exist, but cache is full
    Evicted {
        // key for evicted entry
        key: K,
        // value for evicted entry
        value: V,
    },
    // Only for cache with multiple queues
    EvictedAndUpdate {
        // evicted entry
        evicted: (K, V),
        // old value
        update: V,
    },
}

impl<K: PartialEq, V: PartialEq> PartialEq for PutResult<K, V> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            PutResult::Put => matches!(other, PutResult::Put),
            PutResult::Update(old_val) => match other {
                PutResult::Update(v) => *v == *old_val,
                _ => false,
            },
            PutResult::Evicted { key, value } => match other {
                PutResult::Evicted { key: ok, value: ov } => *key == *ok && *value == *ov,
                _ => false,
            },
            PutResult::EvictedAndUpdate { evicted, update } => match other {
                PutResult::EvictedAndUpdate {
                    evicted: other_evicted,
                    update: other_update,
                } => {
                    evicted.0 == other_evicted.0
                        && evicted.1 == other_evicted.1
                        && *update == *other_update
                }
                _ => false,
            },
        }
    }
}

impl<K: Eq, V: Eq> Eq for PutResult<K, V> {}

impl<K: Debug, V: Debug> Debug for PutResult<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PutResult::Put => write!(f, "PutResult::Put"),
            PutResult::Update(old_val) => write!(f, "PutResult::Update({:?})", *old_val),
            PutResult::Evicted { key: k, value: v } => {
                write!(f, "PutResult::Evicted {{key: {:?}, val: {:?}}}", *k, *v)
            }
            PutResult::EvictedAndUpdate { evicted, update } => write!(f, "PutResult::EvictedAndUpdate {{ evicted: {{key: {:?}, value: {:?}}}, update: {:?} }}", evicted.0, evicted.1, *update),
        }
    }
}

impl<K: Clone, V: Clone> Clone for PutResult<K, V> {
    fn clone(&self) -> Self {
        match self {
            PutResult::Put => PutResult::Put,
            PutResult::Update(v) => PutResult::Update(v.clone()),
            PutResult::Evicted { key: k, value: v } => PutResult::Evicted {
                key: k.clone(),
                value: v.clone(),
            },
            PutResult::EvictedAndUpdate { evicted, update } => PutResult::EvictedAndUpdate {
                evicted: evicted.clone(),
                update: update.clone(),
            },
        }
    }
}

impl<K: Copy, V: Copy> Copy for PutResult<K, V> {}
