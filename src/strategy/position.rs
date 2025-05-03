use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionType {
    Long,
    Short,
    Both,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub position_type: PositionType,
    pub entry_price: f64,
    pub size: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
}

#[derive(Debug, Clone)]
pub enum TradePosition {
    /// A long position:
    /// - `quantity`: how many units/contracts you bought
    /// - `entry_price`: the price at which you opened the position
    /// - `held_bars`: how many time‑period bars you’ve held it
    Long {
        quantity: f64,
        entry_price: f64,
        held_bars: u32,
    },

    /// A short position (you sold first, hoping to buy back cheaper):
    /// same fields as `Long`
    Short {
        quantity: f64,
        entry_price: f64,
        held_bars: u32,
    },
}
