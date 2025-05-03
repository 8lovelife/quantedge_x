use crate::strategy::market_data::MarketData;

pub trait DataFeed {
    fn next(&mut self) -> Option<MarketData>;
    fn reset(&mut self);
}
