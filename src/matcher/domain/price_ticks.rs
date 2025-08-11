use std::fmt::Display;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Decode, Encode,
)]
pub struct PriceTicks(pub i64);

impl PriceTicks {
    pub fn new(raw: i64, tick_size: i64) -> Result<Self, String> {
        if raw % tick_size != 0 {
            return Err(format!("price {} not aligned to tick {}", raw, tick_size));
        }
        Ok(Self(raw))
    }
}

impl Display for PriceTicks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
