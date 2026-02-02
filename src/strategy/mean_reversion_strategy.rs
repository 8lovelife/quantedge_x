use serde_json::Value;

use crate::{
    indicators::{
        indicator::Indicator,
        moving_average::{SmaIndicator, ema_indicator::EmaIndicator, wma_indicator::WmaIndicator},
        std_dev_indicator::StdDevIndicator,
    },
    strategy::position::PositionType,
};

use super::{signal::Signal, strategy_trait::Strategy};

pub struct MeanReversionStrategy {
    name: String,
    mean: Box<dyn Indicator>,
    volatility: Box<dyn Indicator>,
    style: EntryStyle,
    band_multiplier: f64, // Bollinger 带宽倍数
    exit_threshold: f64,

    entry_z_score: f64,
    exit_z_score: f64,

    position_type: PositionType,
}

pub enum EntryStyle {
    ZScore,
    Bollinger,
}

impl MeanReversionStrategy {
    pub fn from_params(params: &Value) -> Box<dyn Strategy> {
        let position_type = match params.get("positionType").and_then(Value::as_str) {
            Some("long") => PositionType::Long,
            Some("short") => PositionType::Short,
            _ => PositionType::Both,
        };

        let mean_type = params
            .get("meanType")
            .and_then(Value::as_str)
            .unwrap_or("sma");
        let period = params
            .get("lookbackPeriod")
            .and_then(Value::as_u64)
            .unwrap_or(20) as usize;
        let style = match params
            .get("reversionStyle")
            .and_then(Value::as_str)
            .unwrap_or("zscore")
        {
            "bollinger" => EntryStyle::Bollinger,
            _ => EntryStyle::ZScore,
        };

        let entry_z_score = params
            .get("entryZScore")
            .and_then(Value::as_f64)
            .unwrap_or(2.0);

        let exit_z_score = params
            .get("exitZScore")
            .and_then(Value::as_f64)
            .unwrap_or(0.5);

        let exit_th = params
            .get("exitThreshold")
            .and_then(Value::as_f64)
            .unwrap_or(0.5);

        let band_mul = params
            .get("bandMultiplier")
            .and_then(Value::as_f64)
            .unwrap_or(2.0);

        let make_mean = |t: &str| -> Box<dyn Indicator> {
            match t {
                "wma" => Box::new(WmaIndicator::new(period)),
                "ema" => Box::new(EmaIndicator::new(period)),
                _ => Box::new(SmaIndicator::new(period)),
            }
        };

        Box::new(MeanReversionStrategy {
            name: "mean-reversion".to_string(),
            mean: make_mean(mean_type),
            volatility: Box::new(StdDevIndicator::new(period)),
            style,
            band_multiplier: band_mul,
            exit_threshold: exit_th,

            entry_z_score: entry_z_score,
            exit_z_score: exit_z_score,
            position_type,
        })
    }
}

impl Strategy for MeanReversionStrategy {
    fn generate_signal(&mut self, price: f64, position: f64) -> Signal {
        self.mean.update(price);
        self.volatility.update(price);

        if let (Some(m), Some(vol)) = (self.mean.value(), self.volatility.value()) {
            match self.style {
                EntryStyle::ZScore => {
                    let z = (price - m) / vol;
                    // 平仓优先
                    if position > 0.0 && z.abs() <= self.exit_z_score {
                        return Signal::Exit;
                    }
                    if position < 0.0 && z.abs() <= self.exit_z_score {
                        return Signal::Exit;
                    }
                    // 入场
                    if z >= self.entry_z_score && position >= 0.0 {
                        return Signal::EnterShort(price);
                    }
                    if z < -self.entry_z_score && position <= 0.0 {
                        return Signal::EnterLong(price);
                    }
                }
                EntryStyle::Bollinger => {
                    let upper = m + self.band_multiplier * vol;
                    let lower = m - self.band_multiplier * vol;
                    // 平仓
                    let upper_exit = m + self.exit_threshold * vol;
                    let lower_exit = m - self.exit_threshold * vol;
                    if position > 0.0 && price >= upper_exit {
                        return Signal::Exit;
                    }
                    if position < 0.0 && price <= lower_exit {
                        return Signal::Exit;
                    }
                    // 入场
                    if price > upper && position >= 0.0 {
                        return Signal::EnterShort(price);
                    }
                    if price < lower && position <= 0.0 {
                        return Signal::EnterLong(price);
                    }
                }
            }
        }
        Signal::Hold
    }

    fn update(
        &mut self,
        market_data: &super::market_data::MarketData,
        current_position: &Option<super::position::TradePosition>,
    ) {
        todo!()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn apply_parameters(&mut self, entry_threshold: Option<f64>, exit_threshold: Option<f64>) {
        todo!()
    }

    fn position_type(&self) -> &PositionType {
        &self.position_type
    }
}
