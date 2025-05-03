use crate::{
    domain::{
        OrderRequest, OrderSide, PositionSizer, TradeObserver, executor::OrderExecutor,
        trade_observer::TradeRecord,
    },
    strategy::{
        direction::Direction, market_data::MarketData, signal::Signal,
        strategy_context::StrategyContext,
    },
};

pub struct SignalProcessor<EX>
where
    EX: OrderExecutor,
{
    executor: EX,
    sizer: Box<dyn PositionSizer>,
    observers: Vec<Box<dyn TradeObserver>>,
}

impl<EX> SignalProcessor<EX>
where
    EX: OrderExecutor,
{
    pub fn new(executor: EX, sizer: Box<dyn PositionSizer>) -> Self {
        Self {
            executor,
            sizer,
            observers: Vec::new(),
        }
    }

    pub fn add_observer(&mut self, o: Box<dyn TradeObserver>) {
        self.observers.push(o);
    }

    /// 同步仓位，从 executor 拉数据
    pub fn sync_positions(&mut self) -> f64 {
        self.executor.sync_positions()
    }

    /// 核心：把 Signal “落地”成下单、状态更新、日志广播
    pub fn process(&mut self, sig: Signal, ctx: &mut StrategyContext, data: &MarketData) {
        // ctx.reconcile_entry();
        match sig {
            Signal::EnterLong(price) if ctx.current_entry.is_none() && ctx.position <= 0.0 => {
                let qty = self.sizer.calc(price, ctx);
                if qty <= 0.0 {
                    return;
                }
                let req = OrderRequest {
                    side: OrderSide::Buy,
                    price,
                    quantity: qty,
                    timestamp: data.timestamp.to_string(),
                };

                let resp = self.executor.execute(req);

                ctx.position += resp.filled_qty;
                ctx.current_entry = Some((
                    resp.timestamp,
                    resp.filled_price,
                    resp.filled_qty,
                    Direction::Long,
                ));
            }
            Signal::EnterShort(price) if ctx.current_entry.is_none() && ctx.position >= 0.0 => {
                let qty = self.sizer.calc(price, ctx);
                if qty <= 0.0 {
                    return;
                }
                let req = OrderRequest {
                    side: OrderSide::Sell,
                    price,
                    quantity: qty,
                    timestamp: data.timestamp.to_string(),
                };

                let resp = self.executor.execute(req);
                ctx.position -= resp.filled_qty;
                ctx.current_entry = Some((
                    resp.timestamp.clone(),
                    resp.filled_price,
                    resp.filled_qty,
                    Direction::Short,
                ));
            }
            Signal::Exit => {
                if let Some((et, entry_price, qty, dir)) = ctx.current_entry.take() {
                    let (side, signed_qty) = if dir == Direction::Long {
                        (OrderSide::Sell, qty)
                    } else {
                        (OrderSide::Buy, qty)
                    };
                    let resp = self.executor.execute(OrderRequest {
                        side,
                        price: data.close_price,
                        quantity: signed_qty,
                        timestamp: data.timestamp.to_string(),
                    });
                    ctx.position += if dir == Direction::Long {
                        -resp.filled_qty
                    } else {
                        resp.filled_qty
                    };

                    let pnl = if dir == Direction::Long {
                        (resp.filled_price - entry_price) * qty
                    } else {
                        (entry_price - resp.filled_price) * qty
                    };
                    let record = TradeRecord {
                        entry_time: et.clone(),
                        exit_time: resp.timestamp.clone(),
                        entry_price,
                        exit_price: resp.filled_price,
                        quantity: qty,
                        direction: dir.clone(),
                        pnl,
                        holding_time: "".to_string(),
                    };
                    // 广播给所有观察者
                    for o in &mut self.observers {
                        o.on_trade(&record);
                    }
                }
            }
            _ => {}
        }
    }
}
