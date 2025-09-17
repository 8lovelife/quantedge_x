pub enum RejectReason {
    FokNotFilled,
    InvalidPrice,
    InvalidQuantity,
    InsufficientBalance,
    Other(String),
}
