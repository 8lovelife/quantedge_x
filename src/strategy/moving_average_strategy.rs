use serde_json::Value;

use crate::indicators::moving_average::{MovingAverage, MovingAverageFactory, MovingAverageType};

use super::{
    market_data::MarketData,
    position::{Position, PositionType, TradePosition},
    signal::Signal,
    strategy_context::StrategyContext,
    strategy_trait::Strategy,
};

pub struct MovingAverageStrategy {
    name: String,
    short_ma: Box<dyn MovingAverage>,
    long_ma: Box<dyn MovingAverage>,
    entry_threshold: Option<f64>,
    exit_threshold: Option<f64>,
    position_type: PositionType,
}

impl MovingAverageStrategy {
    pub fn new(
        name: String,
        short_period: MovingAverageType,
        long_period: MovingAverageType,
    ) -> Self {
        MovingAverageStrategy {
            name,
            short_ma: MovingAverageFactory::create_moving_average(short_period),
            long_ma: MovingAverageFactory::create_moving_average(long_period),
            entry_threshold: None,
            exit_threshold: None,
            position_type: PositionType::Both,
        }
    }
    pub fn from_params(params: &Value) -> Box<dyn Strategy> {
        let position_type = match params.get("positionType").and_then(Value::as_str) {
            Some("long") => PositionType::Long,
            Some("short") => PositionType::Short,
            _ => PositionType::Both,
        };

        let ma_type = params
            .get("maType")
            .and_then(Value::as_str)
            .unwrap_or("sma");

        let fast = params
            .get("fastPeriod")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(5);

        let slow = params
            .get("slowPeriod")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(20);

        let entry_threshold = params.get("entryThreshold").and_then(Value::as_f64);

        let exit_threshold = params.get("exitThreshold").and_then(Value::as_f64);
        Box::new(Self::build(
            "ma-crossover".to_string(),
            ma_type,
            fast,
            slow,
            position_type,
            entry_threshold,
            exit_threshold,
        ))
    }

    pub fn build(
        name: String,
        sub_type: &str,
        fast_period: usize,
        slow_period: usize,
        position_type: PositionType,
        entry_threshold: Option<f64>,
        exit_threshold: Option<f64>,
    ) -> Self {
        let short_ma = match sub_type {
            "sma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::SMA(fast_period))
            }
            "ema" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::EMA(fast_period))
            }
            "wma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::WMA(fast_period))
            }
            _ => panic!("Unsupported moving average sub_type: {}", sub_type),
        };

        let long_ma = match sub_type {
            "sma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::SMA(slow_period))
            }
            "ema" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::EMA(slow_period))
            }
            "wma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::WMA(slow_period))
            }
            _ => panic!("Unsupported moving average sub_type: {}", sub_type),
        };

        MovingAverageStrategy {
            name,
            short_ma,
            long_ma,
            entry_threshold,
            exit_threshold,
            position_type,
        }
    }

    pub fn create(name: String, sub_type: &str, fast_period: usize, slow_period: usize) -> Self {
        let short_ma = match sub_type {
            "sma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::SMA(fast_period))
            }
            "ema" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::EMA(fast_period))
            }
            "wma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::WMA(fast_period))
            }
            _ => panic!("Unsupported moving average sub_type: {}", sub_type),
        };

        let long_ma = match sub_type {
            "sma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::SMA(slow_period))
            }
            "ema" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::EMA(slow_period))
            }
            "wma" => {
                MovingAverageFactory::create_moving_average(MovingAverageType::WMA(slow_period))
            }
            _ => panic!("Unsupported moving average sub_type: {}", sub_type),
        };

        MovingAverageStrategy {
            name,
            short_ma,
            long_ma,
            entry_threshold: None,
            exit_threshold: None,
            position_type: PositionType::Both,
        }
    }
}

impl Strategy for MovingAverageStrategy {
    fn update(&mut self, market_data: &MarketData, _current_position: &Option<TradePosition>) {
        let price = market_data.close_price;
        self.short_ma.update(price);
        self.long_ma.update(price);
    }

    fn generate_signal(&mut self, price: f64, position: f64) -> Signal {
        self.short_ma.update(price);
        self.long_ma.update(price);
        match (self.short_ma.value(), self.long_ma.value()) {
            (Some(short), Some(long)) => {
                let diff = short - long;
                // 多头入场条件：diff >= entry_threshold（或 > 0）
                let want_long = if let Some(et) = self.entry_threshold {
                    diff >= et
                } else {
                    diff > 0.0
                };
                // 空头入场条件：diff <= -entry_threshold（或 < 0）
                let want_short = if let Some(et) = self.entry_threshold {
                    diff <= -et
                } else {
                    diff < 0.0
                };
                // 平仓条件：|diff| <= exit_threshold（或对称于 0）
                let want_exit = if let Some(xet) = self.exit_threshold {
                    diff.abs() <= xet
                } else {
                    false
                };

                // 优先平仓
                if position > 0.0 && want_exit {
                    return Signal::Exit;
                }
                if position < 0.0 && want_exit {
                    return Signal::Exit;
                }
                // 然后入场
                if want_long && position <= 0.0 {
                    return Signal::EnterLong(price);
                }
                if want_short && position >= 0.0 {
                    return Signal::EnterShort(price);
                }

                Signal::Hold
            }
            _ => Signal::Hold, // 任一均线尚未就绪
        }
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn apply_parameters(&mut self, entry_threshold: Option<f64>, exit_threshold: Option<f64>) {
        self.entry_threshold = entry_threshold;
        self.exit_threshold = exit_threshold;
    }

    fn position_type(&self) -> &PositionType {
        &self.position_type
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_moving_average_signals() {
//         let mut strategy = MovingAverageStrategy::new(
//             "MA(5,10)".to_string(),
//             MovingAverageType::SMA(5),
//             MovingAverageType::SMA(10),
//         );

//         // Simulate price movement: flat, then up, then down
//         let prices = vec![
//             10.0, 10.0, 10.0, 10.0, 10.0, // First 5 prices are flat
//             10.0, 10.0, 10.0, 10.0, 10.0, // Complete the 10-period initialization
//             11.0, 12.0, 13.0, 14.0, 15.0, // Rising prices
//             14.0, 13.0, 12.0, 11.0, 10.0, // Falling prices
//         ];

//         let mut signals = Vec::new();

//         for price in prices {
//             let market_data = MarketData {
//                 timestamp: "0".to_string(),
//                 // open_price: price,
//                 // high_price: price,
//                 // low_price: price,
//                 close_price: price,
//                 // volume: 1000,
//             };

//             strategy.update(&market_data, &None);
//             let signal = strategy.generate_signal(&market_data);
//             signals.push(signal);
//         }

//         // First 9 signals should be None (not enough data)
//         for i in 0..9 {
//             assert_eq!(signals[i], Signal::Hold);
//         }

//         // Should eventually see a Buy signal as short MA rises above long MA
//         let buy_detected = signals.iter().skip(10).any(|&s| s == Signal::EnterLong(()));
//         assert!(buy_detected, "Should detect Buy signal when prices rise");

//         // Should eventually see a Sell signal as short MA falls below long MA
//         let sell_detected = signals.iter().skip(15).any(|&s| s == Signal::Sell);
//         assert!(sell_detected, "Should detect Sell signal when prices fall");
//     }
// }
