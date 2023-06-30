use std::collections::HashMap;

use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Store {
    store: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

impl Store {
    pub fn new() -> Self {
        let store = RwLock::new(HashMap::new());
        Store { store }
    }

    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let store = self.store.read().await;
        (*store).get(key).cloned()
    }

    pub async fn set(&self, key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
        let mut store = self.store.write().await;
        (*store).insert(key.into(), value.into())
    }

    pub async fn delete(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut store = self.store.write().await;
        (*store).remove(key)
    }

    pub async fn list(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let store = self.store.read().await;
        return (*store)
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }
}
