use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        order::{Order, OrderSide},
        tif_result::TifResult,
    },
    executor::order_executor::OrderTypeExecutor,
};

pub struct MarketExecutor;

impl<T: OrderBookOps> OrderTypeExecutor<T> for MarketExecutor {
    fn execute(&mut self, order: Order, book: &mut T) -> anyhow::Result<TifResult> {
        match order.side {
            OrderSide::Buy => {
                let (fills, filled) = book.sweep_market_buy(order.qty)?;
                Result::Ok(TifResult::accepted_with_cancel(
                    fills,
                    filled,
                    order.qty - filled,
                ))
            }
            OrderSide::Sell => {
                let (fills, filled) = book.sweep_market_sell(order.qty)?;
                Result::Ok(TifResult::accepted_with_cancel(
                    fills,
                    filled,
                    order.qty - filled,
                ))
            }
        }
    }
}
