#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum RejectReason {
    NoMatchingOrder,
    FokNotFilled,
    Expired,
    InvalidPrice,
    InvalidQuantity,
    InsufficientBalance,
    Other(String),
}
