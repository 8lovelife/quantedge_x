use std::{collections::HashMap, vec};

use crate::{
    domain::order::Side,
    matcher::domain::{
        execution_event::ExecutionEvent,
        order::{Order, OrderSide, OrderType},
        price_ticks::PriceTicks,
        rest_on_book::RestOnBookType,
        tif_policy_result::TifPolicyResult,
    },
};

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub events: Vec<ExecutionEvent>,
    pub prices: HashMap<Side, Vec<PriceTicks>>,
    pub order: Order,
}

impl ExecutionResult {
    pub fn from_tif_result(order: Order, tif_result: TifPolicyResult) -> Self {
        let mut two_way_prices = HashMap::new();
        let mut events = Vec::new();
        let mut prices = Vec::new();
        let order_id = order.id;
        match tif_result {
            TifPolicyResult::Accepted {
                fills,
                completed_order_ids,
                ..
            } => {
                for fill in fills {
                    prices.push(fill.price);
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
                    prices.push(fill.price);
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

        let (bid_prices, ask_prices) = match (order.order_type, order.side) {
            (OrderType::Limit, OrderSide::Buy) => (vec![order.px], prices),
            (OrderType::Limit, OrderSide::Sell) => (prices, vec![order.px]),
            (OrderType::Market, OrderSide::Buy) => (Vec::new(), prices),
            (OrderType::Market, OrderSide::Sell) => (prices, Vec::new()),
        };
        two_way_prices.insert(Side::Ask, ask_prices);
        two_way_prices.insert(Side::Bid, bid_prices);
        Self {
            events,
            order,
            prices: two_way_prices,
        }
    }
}
