mod ema;
pub mod ema_indicator;
mod ma;
mod ma_type;
mod moving_average_factory;
mod sma;
pub mod sma_indicator;
mod wma;
pub mod wma_indicator;

pub use ema::ExponentialMovingAverage;
pub use ma::MovingAverage;
pub use ma_type::MovingAverageType;
pub use moving_average_factory::MovingAverageFactory;
pub use sma::SimpleMovingAverage;
pub use sma_indicator::SmaIndicator;
pub use wma::WeightedMovingAverage;
