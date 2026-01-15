#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Ask,
    Bid,
}

/// 从执行器返回的成交结果
#[derive(Debug, Clone)]
pub struct OrderResponse {
    /// 交易所或系统返回的唯一订单 ID
    pub order_id: String,
    /// 成交方向：Buy / Sell
    pub side: OrderSide,
    /// 成交价格（可能包含滑点）
    pub filled_price: f64,
    /// 成交数量（可能是部分成交）
    pub filled_qty: f64,
    /// 请求时的原始时间戳
    pub timestamp: String,
}
