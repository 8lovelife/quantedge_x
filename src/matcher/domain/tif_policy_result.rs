use crate::matcher::domain::{fill::Fill, qty_lots::QtyLots, rest_on_book::RestOnBook};

pub enum TifPolicyResult {
    Accepted {
        fills: Vec<Fill>,
        filled: QtyLots,
        completed_order_ids: Option<Vec<u64>>,
    },
    AcceptedWithCancel {
        fills: Vec<Fill>,
        filled: QtyLots,
        canceled: QtyLots,
        completed_order_ids: Option<Vec<u64>>,
    },
    AcceptedAndPlaced {
        fills: Vec<Fill>,
        filled: QtyLots,
        rest: RestOnBook,
        completed_order_ids: Option<Vec<u64>>,
    },
    Rejected {
        canceled: QtyLots,
    },
}

impl TifPolicyResult {
    pub fn accepted(
        fills: Vec<Fill>,
        filled: QtyLots,
        completed_order_ids: Option<Vec<u64>>,
    ) -> Self {
        TifPolicyResult::Accepted {
            fills,
            filled,
            completed_order_ids,
        }
    }

    pub fn accepted_with_cancel(
        fills: Vec<Fill>,
        filled: QtyLots,
        canceled: QtyLots,
        completed_order_ids: Option<Vec<u64>>,
    ) -> Self {
        TifPolicyResult::AcceptedWithCancel {
            fills,
            filled,
            canceled,
            completed_order_ids,
        }
    }

    pub fn accepted_and_placed(
        fills: Vec<Fill>,
        filled: QtyLots,
        rest: RestOnBook,
        completed_order_ids: Option<Vec<u64>>,
    ) -> Self {
        TifPolicyResult::AcceptedAndPlaced {
            fills,
            filled,
            rest,
            completed_order_ids,
        }
    }

    pub fn rejected(canceled: QtyLots) -> Self {
        TifPolicyResult::Rejected { canceled }
    }
}
