use std::collections::HashMap;

use duckdb::arrow::array::StringBuilder;
use serde_json::Value;

use crate::{
    api::handlers::backtest::StrategyParams, indicators::moving_average::MovingAverageType,
};

use super::{
    mean_reversion_strategy::MeanReversionStrategy,
    moving_average_strategy::MovingAverageStrategy,
    strategy_trait::Strategy,
    strategy_type::{StrategyType, SupportStrategyType},
};

type StrategyBuilder = fn(&Value) -> Box<dyn Strategy>;

pub struct StrategyFactory {
    registry: HashMap<String, StrategyBuilder>,
}

impl StrategyFactory {
    pub fn new() -> Self {
        let mut factory = StrategyFactory {
            registry: HashMap::new(),
        };

        factory.registry("ma-crossover", MovingAverageStrategy::from_params);
        factory.registry("mean-reversion", MeanReversionStrategy::from_params);

        factory
    }

    pub fn registry(&mut self, name: &str, builder: StrategyBuilder) {
        self.registry.insert(name.to_string(), builder);
    }

    pub fn build(&self, name: &str, params: &Value) -> Option<Box<dyn Strategy>> {
        self.registry.get(name).map(|b| b(params))
    }

    pub fn create_strategy(name: String, strategy_type: StrategyType) -> Box<dyn Strategy> {
        match strategy_type {
            StrategyType::MA {
                short_period,
                long_period,
            } => {
                let strategy = MovingAverageStrategy::new(name, short_period, long_period);

                Box::new(strategy)
            }
        }
        // let strategy = MovingAverageStrategy::new(
        //     name,
        //     MovingAverageType::EMA(short_period),
        //     MovingAverageType::EMA(long_period),
        // );
        // Box::new(strategy)
    }

    pub fn create(
        name: String,
        r#type: &str,   // e.g., "MA Crossover"
        sub_type: &str, // e.g., "sma" or "ema"
        fast_period: u32,
        slow_period: u32,
    ) -> Box<dyn Strategy> {
        match r#type.parse().unwrap() {
            SupportStrategyType::MovingAverageCrossover => Box::new(MovingAverageStrategy::create(
                name,
                sub_type,
                fast_period as usize,
                slow_period as usize,
            )),
            SupportStrategyType::RSI => {
                panic!("RSI strategy not implemented yet");
            }
            _ => panic!("Unsupported strategy type"),
        }
    }
}
