//! # AwaitState
//!
//! A minimal, powerful async state coordination map for Rust — wait for state changes, not just values.
//! **`AwaitState`** lets you insert state into a concurrent map and await custom transitions using user-defined predicates. Built on `tokio` and `dashmap`, it offers a simple but robust way to coordinate async tasks by state — with support for previous/current value tracking and notification on change.
//! ---

//! ## Features

//! - Track both **previous and current state**
//! - Wait for **custom state transitions** using predicates
//! - Efficient notification without polling (uses `tokio::Notify`)
//! - Backed by `DashMap` for thread-safe concurrent access
//! - Lightweight, no macros, no codegen

//! ---

//! ## Example

//! ```rust
//! use awaitstate::AwaitStateMap;
//! use std::sync::Arc;
//! use tokio::time::Duration;

//! #[derive(Clone, Debug, PartialEq)]
//! enum DownloadState {
//!     NotStarted,
//!     Started,
//!     Finished,
//!     Error,
//! }

//! #[tokio::main]
//! async fn main() {
//!     let map = Arc::new(AwaitStateMap::new());
//!     map.put("download_1", DownloadState::NotStarted);

//!     // Simulate async progress in a background task
//!     let map_clone = map.clone();
//!     tokio::spawn(async move {
//!         map_clone.set_state("download_1", DownloadState::Started).await.unwrap();
//!         tokio::time::sleep(Duration::from_secs(1)).await;
//!         map_clone.set_state("download_1", DownloadState::Finished).await.unwrap();
//!     });

//!     // Wait for the state to become Finished
//!     map.wait_until("download_1", |_prev, curr| *curr == DownloadState::Finished)
//!         .await
//!         .unwrap();

//!     println!("Download finished!");
//! }

mod await_state;
mod error;
mod watch_diff;

pub use await_state::AwaitStateMap;
pub use error::AwaitStateError;

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use crate::await_state::AwaitStateMap;

    #[derive(Clone, Debug, PartialEq)]
    enum DownloadState {
        NotStarted,
        Started,
        Finished,
    }

    #[tokio::test]
    async fn test_wait_until() {
        let map = Arc::new(AwaitStateMap::new());
        map.put("download_1", DownloadState::NotStarted);

        let map_move = Arc::clone(&map);
        tokio::spawn(async move {
            map_move
                .set_state("download_1", DownloadState::Started)
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            map_move
                .set_state("download_1", DownloadState::Finished)
                .await
                .unwrap();
        });

        map.wait_until("download_1", |_prev, curr| *curr == DownloadState::Finished)
            .await
            .unwrap();

        assert_eq!(
            map.get_state("download_1").await.unwrap(),
            DownloadState::Finished
        );
    }

    #[tokio::test]
    async fn test_wait_until_timeout() {
        let map = Arc::new(AwaitStateMap::new());
        map.put("download_1", DownloadState::NotStarted);

        let res = map
            .wait_until_timeout(
                "download_1",
                |_prev, curr| *curr == DownloadState::Started,
                Duration::from_millis(100),
            )
            .await;

        assert!(res.is_err());
    }
}
