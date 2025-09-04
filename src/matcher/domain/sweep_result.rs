use crate::matcher::domain::{fill::Fill, qty_lots::QtyLots};

pub enum SweepStatus {
    Full,
    Partial,
    None,
}

pub struct SweepResult {
    pub fills: Vec<Fill>,
    pub filled: QtyLots,
    pub want: QtyLots,
    pub leftover: QtyLots,
    pub status: SweepStatus,
}

impl SweepResult {
    pub fn build(fills: Vec<Fill>, filled: QtyLots, want: QtyLots) -> Self {
        let status = if filled.0 == 0 {
            SweepStatus::None
        } else if filled == want {
            SweepStatus::Full
        } else {
            SweepStatus::Partial
        };

        let left_over = want - filled;
        SweepResult {
            fills,
            filled,
            want,
            leftover: left_over,
            status,
        }
    }
}
