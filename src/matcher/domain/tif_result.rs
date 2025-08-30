use chrono::{DateTime, Utc};

use crate::matcher::domain::{
    fill::Fill, order::OrderSide, price_ticks::PriceTicks, qty_lots::QtyLots,
    rest_on_book::RestOnBook,
};

pub enum TifStatus {
    Accepted,
    Rejected,
}
pub struct TifResult {
    pub fills: Vec<Fill>,
    pub filled: QtyLots,
    pub canceled: Option<QtyLots>,
    pub status: TifStatus,
    pub rest: Option<RestOnBook>,
}

impl TifResult {
    pub fn new(fills: Vec<Fill>, filled: QtyLots, status: TifStatus) -> Self {
        Self {
            fills,
            filled,
            canceled: None,
            status,
            rest: None,
        }
    }

    pub fn rejected_with_cancel(requested: QtyLots) -> Self {
        Self {
            status: TifStatus::Rejected,
            fills: Vec::new(),
            filled: QtyLots(0),
            canceled: Some(requested),
            rest: None,
        }
    }

    pub fn rejected() -> Self {
        Self {
            status: TifStatus::Rejected,
            fills: Vec::new(),
            filled: QtyLots(0),
            canceled: None,
            rest: None,
        }
    }

    pub fn accepted(fills: Vec<Fill>, filled: QtyLots) -> Self {
        Self::from_fills(TifStatus::Accepted, fills, filled)
    }

    pub fn accepted_with_cancel(fills: Vec<Fill>, filled: QtyLots, canceled: QtyLots) -> Self {
        let mut result = Self::from_fills(TifStatus::Accepted, fills, filled);
        result.with_cancel(canceled);
        result
    }

    fn from_fills(status: TifStatus, fills: Vec<Fill>, filled: QtyLots) -> Self {
        Self {
            fills,
            filled,
            canceled: None,
            status,
            rest: None,
        }
    }

    pub fn with_cancel(&mut self, canceled: QtyLots) {
        self.canceled = Some(canceled)
    }

    pub fn with_rest(
        &mut self,
        side: OrderSide,
        limit: PriceTicks,
        rest_qty: QtyLots,
        expires_at: Option<DateTime<Utc>>,
    ) {
        let rest = Some(RestOnBook {
            side,
            limit,
            qty: rest_qty,
            expires_at,
        });
        self.rest = rest;
    }
}
