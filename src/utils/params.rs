use serde::Serialize;
use serde_json::{Map, Value};

pub struct ParamSchema {
    pub strategy_keys: &'static [&'static str],
    pub risk_keys: &'static [&'static str],
    pub exec_keys: &'static [&'static str],
}

#[derive(Serialize)]
pub struct SplitParams {
    pub strategy: Value,
    pub risk: Value,
    pub exec: Value,
}

pub fn split_params(all: &Value, name: &str) -> SplitParams {
    let schema = schema_for_strategy(name);
    let mut strat_map = Map::new();
    let mut risk_map = Map::new();
    let mut exec_map = Map::new();

    if let Some(obj) = all.as_object() {
        for (k, v) in obj {
            if schema.strategy_keys.contains(&k.as_str()) {
                strat_map.insert(k.clone(), v.clone());
            }
            if schema.risk_keys.contains(&k.as_str()) {
                risk_map.insert(k.clone(), v.clone());
            }
            if schema.exec_keys.contains(&k.as_str()) {
                exec_map.insert(k.clone(), v.clone());
            }
        }
    }

    SplitParams {
        strategy: Value::Object(strat_map),
        risk: Value::Object(risk_map),
        exec: Value::Object(exec_map),
    }
}

pub fn schema_for_strategy(name: &str) -> ParamSchema {
    match name {
        "ma-crossover" => ParamSchema {
            strategy_keys: &[
                "meanType",
                "fastPeriod",
                "slowPeriod",
                "entryThreshold",
                "exitThreshold",
                "rebalanceInterval",
                "entryDelay",
            ],
            risk_keys: &[
                "stopLoss",
                "takeProfit",
                "riskPerTrade",
                "maxConcurrentPositions",
                "positionSize",
            ],
            exec_keys: &["slippage", "commission"],
        },
        "mean-reversion" => ParamSchema {
            strategy_keys: &[
                "meanType",
                "lookbackPeriod",
                "entryZScore",
                "exitZScore",
                "reversionStyle",
                "bandMultiplier",
                "exitThreshold",
            ],
            risk_keys: &[
                "stopLoss",
                "takeProfit",
                "riskPerTrade",
                "maxConcurrentPositions",
                "positionSize",
            ],
            exec_keys: &["slippage", "commission"],
        },
        _ => ParamSchema {
            strategy_keys: &[],
            risk_keys: &[],
            exec_keys: &[],
        },
    }
}
