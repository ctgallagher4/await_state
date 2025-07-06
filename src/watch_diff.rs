use tokio::sync::{Notify, RwLock};

struct Inner<T> {
    prev: Option<T>,
    current: T,
}

pub struct WatchDiff<T> {
    inner: RwLock<Inner<T>>,
    notify: Notify,
}

impl<T: Clone + PartialEq> WatchDiff<T> {
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

    pub async fn set(&self, new: T) {
        let mut write = self.inner.write().await;
        let current = write.current.clone();
        write.prev = Some(current);
        write.current = new;
        self.notify.notify_waiters();
    }

    pub async fn get_diff_cloned(&self) -> (Option<T>, T) {
        let read = self.inner.read().await;
        (read.prev.clone(), read.current.clone())
    }

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
