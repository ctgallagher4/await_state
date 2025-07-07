use tokio::sync::{Notify, RwLock};

/// A struct to store previous and current state
struct Inner<T> {
    prev: Option<T>,
    current: T,
}

/// A struct to store the inner state and Tokio notify
pub struct WatchDiff<T> {
    inner: RwLock<Inner<T>>,
    notify: Notify,
}

impl<T: Clone + PartialEq> WatchDiff<T> {
    /// Creates a new WatchDiff with an initial state
    pub fn new(initial: T) -> Self {
        let inner = Inner {
            prev: None,
            current: initial,
        };
        let watch_diff = WatchDiff {
            inner: RwLock::new(inner),
            notify: Notify::new(),
        };
        watch_diff
    }

    /// Sets WatchDiff State
    pub async fn set(&self, new: T) {
        let mut write = self.inner.write().await;
        let current = write.current.clone();
        write.prev = Some(current);
        write.current = new;
        self.notify.notify_waiters();
    }

    /// Get the past and current state with cloning
    pub async fn get_diff_cloned(&self) -> (Option<T>, T) {
        let read = self.inner.read().await;
        (read.prev.clone(), read.current.clone())
    }

    /// Check if the state has changed and return it if so.
    pub async fn changed(&self) -> (T, T) {
        loop {
            self.notify.notified().await;
            let (prev, curr) = self.get_diff_cloned().await;
            if let Some(prev) = prev {
                if curr != prev {
                    return (prev, curr);
                }
            }
        }
    }
}
