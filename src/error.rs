#[derive(Debug, thiserror::Error)]
pub enum AwaitStateError {
    #[error("key not found")]
    KeyNotFound,

    #[error("timeout expired")]
    TimeoutExpired,
}
