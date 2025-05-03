use std::fmt;

use super::{
    ExponentialMovingAverage, MovingAverage, SimpleMovingAverage, WeightedMovingAverage,
    ma_type::MovingAverageType,
};

pub struct MovingAverageFactory;

impl fmt::Display for MovingAverageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovingAverageType::SMA(period) => write!(f, "SMA({period})"),
            MovingAverageType::EMA(period) => write!(f, "EMA({period})"),
            MovingAverageType::WMA(period) => write!(f, "WMA({period})"),
        }
    }
}

impl MovingAverageFactory {
    // Define a factory function to create different moving averages
    pub fn create_moving_average(ma_type: MovingAverageType) -> Box<dyn MovingAverage> {
        match ma_type {
            MovingAverageType::SMA(period) => Box::new(SimpleMovingAverage::new(period)),
            MovingAverageType::EMA(period) => Box::new(ExponentialMovingAverage::new(period)),
            MovingAverageType::WMA(period) => Box::new(WeightedMovingAverage::new(period)),
        }
    }
}
