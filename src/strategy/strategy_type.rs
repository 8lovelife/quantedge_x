use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::indicators::moving_average::MovingAverageType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    MA {
        short_period: MovingAverageType,
        long_period: MovingAverageType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportStrategyType {
    MovingAverageCrossover,
    RSI,
    BollingerBands,
    MACD,
}

impl SupportStrategyType {
    pub fn all_strategies() -> Vec<SupportStrategyType> {
        vec![
            SupportStrategyType::MovingAverageCrossover,
            SupportStrategyType::RSI,
            SupportStrategyType::BollingerBands,
            SupportStrategyType::MACD,
        ]
    }
}

impl fmt::Display for SupportStrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            SupportStrategyType::MovingAverageCrossover => "MA Crossover",
            SupportStrategyType::RSI => "RSI",
            SupportStrategyType::BollingerBands => "Bollinger Bands",
            SupportStrategyType::MACD => "MACD",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for SupportStrategyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MA Crossover" | "ma" | "ma-crossover" => {
                Ok(SupportStrategyType::MovingAverageCrossover)
            }
            "RSI" | "rsi" => Ok(SupportStrategyType::RSI),
            "BollingerBands" | "bollinger_bands" => Ok(SupportStrategyType::BollingerBands),
            "MACD" | "macd" => Ok(SupportStrategyType::MACD),
            _ => Err(format!("Unknown strategy type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_strategies_output() {
        let strategies = SupportStrategyType::all_strategies();
        for strategy in strategies {
            println!("{:?}", strategy);
        }
    }
}
