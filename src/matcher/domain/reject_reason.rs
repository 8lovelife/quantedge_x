#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum RejectReason {
    NoMatchingOrder,
    FokNotFilled,
    Expired,
    InvalidPrice,
    InvalidQuantity,
    InsufficientBalance,
    Other(String),
}
