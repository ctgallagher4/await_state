# AwaitState

A minimal, powerful async state coordination map for Rust — wait for state changes, not just values.

**`AwaitState`** lets you insert state into a concurrent map and await custom transitions using user-defined predicates. Built on `tokio` and `dashmap`, it offers a simple but robust way to coordinate async tasks by state — with support for previous/current value tracking and notification on change.

---

## Features

- Track both **previous and current state**
- Wait for **custom state transitions** using predicates
- Efficient notification without polling (uses `tokio::Notify`)
- Backed by `DashMap` for thread-safe concurrent access
- Lightweight, no macros, no codegen — just a few hundred LOC

---

## Example

```rust
use await_state::AwaitStateMap;
use std::sync::Arc;
use tokio::time::Duration;

#[derive(Clone, Debug, PartialEq)]
enum DownloadState {
    NotStarted,
    Started,
    Finished,
    Error,
}

#[tokio::main]
async fn main() {
    let map = Arc::new(AwaitStateMap::new());
    map.put("download", DownloadState::NotStarted);

    // Simulate async progress in a background task
    let map_clone = map.clone();
    tokio::spawn(async move {
        map_clone.set_state("download", DownloadState::Started).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        map_clone.set_state("download", DownloadState::Finished).await.unwrap();
    });

    // Wait for the state to become Finished
    map.wait_until("download", |_prev, curr| *curr == DownloadState::Finished)
        .await
        .unwrap();

    println!("Download finished!");
}
```

---

## Use Cases

  * Download manager state tracking

  * Distributed job coordination

  * Reactive actor models

  * Awaitable per-key state transitions

  * Async workflows with checkpointing

---

## Status

**AwaitState** is small, focused, and production-ready. It's designed for projects that want to coordinate async work with as little boilerplate as possible.

---

## License

MIT or Apache 2.0 -- your choice

---

## Contributing

Contributions, feedback, and design discussions are welcome!
