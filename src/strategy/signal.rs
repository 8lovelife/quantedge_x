#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    EnterShort(f64),
    EnterLong(f64),
    Exit,
    Hold,
}
