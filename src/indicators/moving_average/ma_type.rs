use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MovingAverageType {
    SMA(usize),
    EMA(usize),
    WMA(usize),
}
