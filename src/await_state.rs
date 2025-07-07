use dashmap::DashMap;
use std::{sync::Arc, time::Duration};
use tokio::time::timeout;

use crate::{error::AwaitStateError, watch_diff::WatchDiff};

/// A map like struct based on Dashmap which stores state values
/// with a string key.
pub struct AwaitStateMap<T> {
    map: DashMap<String, Arc<WatchDiff<T>>>,
}

impl<T: Clone + PartialEq> AwaitStateMap<T> {
    /// Creates a new AwaitStateMap
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    /// Creates a new AwaitStateMap with a specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: DashMap::with_capacity(capacity),
        }
    }

    /// Puts a state into the map
    pub fn put(&self, key: &str, value: T) {
        let inserted_value = Arc::new(WatchDiff::new(value));
        self.map.insert(key.to_string(), inserted_value);
    }

    /// Removes a state from the map
    pub fn remove(&self, key: &str) {
        self.map.remove(key);
    }

    /// Sets a state in the map
    pub async fn set_state(&self, key: &str, state: T) -> Result<(), AwaitStateError> {
        if let Some(value) = self.map.get(key) {
            value.set(state).await;
            Ok(())
        } else {
            Err(AwaitStateError::KeyNotFound)
        }
    }

    /// Gets a state from the map
    pub async fn get_state(&self, key: &str) -> Result<T, AwaitStateError> {
        if let Some(value) = self.map.get(key) {
            Ok(value.get_diff_cloned().await.1)
        } else {
            Err(AwaitStateError::KeyNotFound)
        }
    }

    /// Waits until a given predicate is true for some state
    pub async fn wait_until<F>(&self, key: &str, predicate: F) -> Result<T, AwaitStateError>
    where
        F: Fn(&T, &T) -> bool + Send + Sync + 'static,
        T: Clone + PartialEq + Send + Sync + 'static,
    {
        loop {
            let entry = self.map.get(key);
            if let Some(entry) = entry {
                let (prev, curr) = entry.get_diff_cloned().await;
                if let Some(prev) = prev.as_ref() {
                    if predicate(prev, &curr) {
                        return Ok(curr);
                    }
                } else {
                    if predicate(&curr, &curr) {
                        return Ok(curr);
                    }
                }
                let (prev, curr) = entry.changed().await;
                if predicate(&prev, &curr) {
                    return Ok(curr);
                }
            } else {
                return Err(AwaitStateError::KeyNotFound);
            }
        }
    }

    /// Waits until a given predicate is true for some state or until a timeout period is over.
    pub async fn wait_until_timeout<F>(
        &self,
        key: &str,
        predicate: F,
        duration: Duration,
    ) -> Result<T, AwaitStateError>
    where
        F: Fn(&T, &T) -> bool + Send + Sync + 'static,
        T: Clone + PartialEq + Send + Sync + 'static,
    {
        timeout(duration, self.wait_until(key, predicate))
            .await
            .map_err(|_| AwaitStateError::TimeoutExpired)?
    }
}
