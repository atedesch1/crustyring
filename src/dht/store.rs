use std::{collections::HashMap, vec};

use tokio::sync::RwLock;

use crate::error::Result;

#[derive(Debug)]
pub struct Store {
    store: RwLock<HashMap<u64, Vec<u8>>>,
}

impl Store {
    pub fn new() -> Self {
        let store = RwLock::new(HashMap::new());
        Store { store }
    }

    pub async fn get(&self, key: &u64) -> Option<Vec<u8>> {
        let store = self.store.read().await;
        (*store).get(key).cloned()
    }

    pub async fn set(&self, key: &u64, value: &[u8]) -> Option<Vec<u8>> {
        let mut store = self.store.write().await;
        (*store).insert(*key, value.into())
    }

    pub async fn delete(&self, key: &u64) -> Option<Vec<u8>> {
        let mut store = self.store.write().await;
        (*store).remove(key)
    }

    pub async fn list(&self) -> Vec<(u64, Vec<u8>)> {
        let store = self.store.read().await;
        (*store)
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub async fn get_entries_satisfy<F>(&self, f: F) -> Vec<(u64, Vec<u8>)>
    where
        F: Fn(u64) -> bool,
    {
        let store = self.store.read().await;

        (*store)
            .iter()
            .filter(|(&ref key, _)| f(*key))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
