use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Decode, Encode,
)]
pub struct QtyLots(pub i64);

impl QtyLots {
    pub fn new(raw: i64, lot_size: i64) -> Result<Self, String> {
        if raw % lot_size != 0 {
            return Err(format!("qty {} not aligned to lot {}", raw, lot_size));
        }
        Ok(Self(raw))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
impl Display for QtyLots {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for QtyLots {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        QtyLots(self.0 + rhs.0)
    }
}
impl Sub for QtyLots {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        QtyLots(self.0 - rhs.0)
    }
}
impl AddAssign for QtyLots {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl SubAssign for QtyLots {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
