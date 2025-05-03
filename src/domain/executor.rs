use super::{OrderRequest, order::OrderResponse};

pub trait OrderExecutor {
    fn execute(&mut self, order: OrderRequest) -> OrderResponse;
    fn sync_positions(&mut self) -> f64;
}
