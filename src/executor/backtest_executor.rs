use chrono::Utc;

use crate::domain::{OrderRequest, OrderSide, executor::OrderExecutor, order::OrderResponse};

/// 回测执行器：模拟市价立即成交，支持滑点和佣金
pub struct BacktestExecutor {
    /// 当前持仓数量
    pub position: f64,
    /// 当前账户现金
    pub cash: f64,
    /// 滑点比例，例如0.001表示0.1%
    pub slippage: f64,
    /// 佣金比例，例如0.0005表示0.05%
    pub commission: f64,
}

impl BacktestExecutor {
    /// 创建新的 BacktestExecutor
    pub fn new(initial_capital: f64) -> Self {
        Self {
            position: 0.0,
            cash: initial_capital,
            slippage: 0.0,
            commission: 0.0,
        }
    }

    /// 设置滑点
    pub fn with_slippage(mut self, slippage: f64) -> Self {
        self.slippage = slippage;
        self
    }

    /// 设置佣金
    pub fn with_commission(mut self, commission: f64) -> Self {
        self.commission = commission;
        self
    }
}

impl OrderExecutor for BacktestExecutor {
    /// 执行订单：模拟市价/限价/止损统一按市价立即成交
    fn execute(&mut self, req: OrderRequest) -> OrderResponse {
        // 计算执行价格：考虑滑点
        let base_price = req.price;
        let exec_price = match req.side {
            OrderSide::Buy => base_price * (1.0 + self.slippage),
            OrderSide::Sell => base_price * (1.0 - self.slippage),
        };

        // 计算成交数量及成本
        let qty = req.quantity;
        let cost = exec_price * qty;
        // 佣金
        let fee = cost * self.commission;

        // 更新持仓和现金
        match req.side {
            OrderSide::Buy => {
                self.position += qty;
                self.cash -= cost + fee;
            }
            OrderSide::Sell => {
                self.position -= qty;
                self.cash += cost - fee;
            }
        }

        // 返回成交回报
        OrderResponse {
            order_id: format!("BT-{}-{:?}", Utc::now().timestamp_millis(), req.side),
            side: req.side,
            filled_price: exec_price,
            filled_qty: qty,
            timestamp: req.timestamp,
        }
    }

    /// 同步持仓
    fn sync_positions(&mut self) -> f64 {
        self.position
    }

    // /// 同步账户净值（仓位转市价 + 现金）
    // fn sync_equity(&mut self) -> f64 {
    //     // equity = cash + position * last_price。
    //     // 对于回测，这里只返回 cash + position*req.price? 需要外部传入价格。
    //     // 简化：只返回 cash
    //     self.cash
    // }
}
