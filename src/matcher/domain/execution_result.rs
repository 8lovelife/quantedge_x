use crate::{
    matcher::domain::{
        execution_event::ExecutionEvent,
        order::Order,
        rest_on_book::RestOnBookType,
        tif_policy_result::TifPolicyResult,
    },
    models::order_book_message::OrderBookMessage,
};

#[derive(Debug)]
pub struct ExecutionResult {
    pub events: Vec<ExecutionEvent>,
    pub order: Order,
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

                if rest.rest_type == RestOnBookType::PartialRest {
                    events.push(ExecutionEvent::Placed {
                        order_id: Some(order_id),
                        qty: rest.qty,
                        price: order.px,
                        expires_at: None,
                    });
                }
            }

            TifPolicyResult::Rejected { reject_reason, .. } => {
                events.push(ExecutionEvent::Rejected {
                    order_id,
                    reason: reject_reason,
                });
            }
        }

        Self { events, order }
    }

    pub fn to_delta_message(&self, start_id: u64) -> Option<OrderBookMessage> {
        // let mut bids = Vec::new();
        // let mut asks = Vec::new();
        // let mut end_id = start_id;
        // let price = &self.order.px;
        // for evt in self.events {
        //     match evt {
        //         ExecutionEvent::Placed {
        //             order_id,
        //             qty,
        //             price,
        //             ..
        //         } => {
        //             match self.order.side {
        //                 OrderSide::Buy => bids.push((price, qty)),
        //                 OrderSide::Sell => asks.push((price, qty)),
        //             }
        //             end_id = std::cmp::max(end_id, order_id.unwrap_or(0));
        //         }
        //         ExecutionEvent::Traded {
        //             taker_order_id,
        //             maker_order_id,
        //             qty,
        //             price,
        //             ..
        //         } => {
        //             // Taker
        //             bids.push((price, qty));
        //             // Maker
        //             asks.push((price, qty));
        //         }
        //         ExecutionEvent::Cancelled {
        //             order_id,
        //             cancelled,
        //             ..
        //         } => {
        //             let qty = cancelled.negative();
        //             match self.order.side {
        //                 OrderSide::Buy => bids.push((price, &qty)),
        //                 OrderSide::Sell => asks.push((price, &qty)),
        //             }
        //         }

        //         ExecutionEvent::Rejected { .. } => {}
        //     }
        // }

        None
    }
}
