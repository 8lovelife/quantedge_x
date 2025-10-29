use chrono::{DateTime, Utc};

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        order::OrderSide,
        price_ticks::PriceTicks,
        qty_lots::QtyLots,
        rest_on_book::{RestOnBook, RestOnBookType},
        sweep_result::SweepResult,
        tif_policy_result::TifPolicyResult,
    },
    policy::tif::tif_policy::TifPolicy,
};

pub struct GttPolicy {
    pub expires_at: DateTime<Utc>,
}

impl TifPolicy for GttPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult> {
        let limit = limit.expect("GTT buy must have a limit price");
        let sweep_result = book.sweep_asks_up_to(limit, want)?;
        match sweep_result {
            SweepResult::None { want } => Ok(TifPolicyResult::accepted_and_placed(
                vec![],
                QtyLots(0),
                RestOnBook {
                    side: OrderSide::Buy,
                    limit,
                    qty: want,
                    rest_type: RestOnBookType::AllRest,
                    expires_at: Some(self.expires_at),
                },
                None,
            )),
            SweepResult::Partial {
                fills,
                filled,
                leftover,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted_and_placed(
                fills,
                filled,
                RestOnBook {
                    side: OrderSide::Buy,
                    limit,
                    qty: leftover,
                    rest_type: RestOnBookType::PartialRest,
                    expires_at: Some(self.expires_at),
                },
                Some(completed_order_ids),
            )),
            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted(
                fills,
                filled,
                Some(completed_order_ids),
            )),
        }
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult> {
        let limit = limit.expect("GTT sell must have a limit price");
        let sweep_result = book.sweep_bids_down_to(limit, want)?;
        match sweep_result {
            SweepResult::None { want } => Ok(TifPolicyResult::accepted_and_placed(
                vec![],
                QtyLots(0),
                RestOnBook {
                    side: OrderSide::Sell,
                    limit,
                    qty: want,
                    rest_type: RestOnBookType::AllRest,
                    expires_at: Some(self.expires_at),
                },
                None,
            )),
            SweepResult::Partial {
                fills,
                filled,
                leftover,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted_and_placed(
                fills,
                filled,
                RestOnBook {
                    side: OrderSide::Sell,
                    limit,
                    qty: leftover,
                    rest_type: RestOnBookType::PartialRest,
                    expires_at: Some(self.expires_at),
                },
                Some(completed_order_ids),
            )),
            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted(
                fills,
                filled,
                Some(completed_order_ids),
            )),
        }
    }
}
