use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderBookSnapshotError {
    #[error("orderbook snapshot not found")]
    NotFound,

    #[error("orderbook snapshot corrupted")]
    Corrupted,

    #[error("io error")]
    Io(#[from] std::io::Error),
}
