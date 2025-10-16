use crate::matcher::domain::{
    execution_event::ExecutionEvent, order::Order, price_ticks::PriceTicks,
    reject_reason::RejectReason, tif_policy_result::TifPolicyResult,
};

#[derive(Debug)]
pub struct ExecutionResult {
    pub events: Vec<ExecutionEvent>,
}

impl ExecutionResult {
    pub fn from_tif_result(order: Order, tif_result: TifPolicyResult) -> Self {
        let mut events = Vec::new();
        let order_id = order.id;
        match tif_result {
            TifPolicyResult::Accepted {
                fills,
                completed_order_ids,
                ..
            } => {
                for fill in fills {
                    events.push(ExecutionEvent::Traded {
                        taker_order_id: order_id,
                        taker_completed: true,

                        maker_order_id: fill.order_id,
                        qty: fill.qty,
                        price: fill.price,
                        maker_completed: completed_order_ids
                            .as_ref()
                            .map(|ids| ids.contains(&order_id))
                            .unwrap_or(false),
                    });
                }
            }

            TifPolicyResult::AcceptedWithCancel {
                fills,
                canceled,
                completed_order_ids,
                ..
            } => {
                for fill in fills {
                    events.push(ExecutionEvent::Traded {
                        taker_order_id: order_id,
                        taker_completed: false,
                        maker_order_id: fill.order_id,
                        qty: fill.qty,
                        price: fill.price,
                        maker_completed: completed_order_ids
                            .as_ref()
                            .map(|ids| ids.contains(&order_id))
                            .unwrap_or(false),
                    });
                }

                events.push(ExecutionEvent::Cancelled {
                    order_id,
                    cancelled: canceled,
                    fully_cancelled: true,
                });
            }

            TifPolicyResult::AcceptedAndPlaced {
                fills,
                rest,
                completed_order_ids,
                ..
            } => {
                for fill in fills {
                    events.push(ExecutionEvent::Traded {
                        taker_order_id: order_id,
                        taker_completed: false,
                        maker_order_id: fill.order_id,
                        qty: fill.qty,
                        price: fill.price,
                        maker_completed: completed_order_ids
                            .as_ref()
                            .map(|ids| ids.contains(&order_id))
                            .unwrap_or(false),
                    });
                }

                events.push(ExecutionEvent::Placed {
                    order_id: None,
                    qty: rest.qty,
                    price: order.px,
                    expires_at: None,
                });
            }

            TifPolicyResult::Rejected { reject_reason, .. } => {
                events.push(ExecutionEvent::Rejected {
                    order_id,
                    reason: reject_reason,
                });
            }
        }

        Self { events }
    }
}
