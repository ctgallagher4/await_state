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
}
