use itertools::Itertools;

use crate::{api::handlers::backtest::GridParams, engine::parameters::StrategyRunParameters};

pub struct OptimizeService {}

impl OptimizeService {
    pub fn grid_search(&self, grid_params: &GridParams) -> Result<(), Box<dyn std::error::Error>> {
        let run_parameters = generate_combos(grid_params);

        Ok(())
    }
}

fn to_vec_opt<T: Clone>(opt: &Option<Vec<T>>) -> Vec<Option<T>> {
    match opt {
        Some(vs) if !vs.is_empty() => vs.iter().cloned().map(Some).collect(),
        _ => vec![None],
    }
}

fn generate_combos(grid: &GridParams) -> Vec<StrategyRunParameters> {
    let fasts = to_vec_opt(&grid.fast_period);
    let slows = to_vec_opt(&grid.slow_period);
    let entry_ths = to_vec_opt(&grid.entry_threshold);
    let exit_ths = to_vec_opt(&grid.exit_threshold);
    let stop_losses = to_vec_opt(&grid.stop_loss);
    let take_profits = to_vec_opt(&grid.take_profit);
    let position_sizes = to_vec_opt(&grid.position_size);

    fasts
        .into_iter()
        .cartesian_product(slows)
        .cartesian_product(entry_ths)
        .cartesian_product(exit_ths)
        .cartesian_product(stop_losses)
        .cartesian_product(take_profits)
        .cartesian_product(position_sizes)
        .map(
            |((((((fast, slow), entry), exit), sl), tp), size)| StrategyRunParameters {
                position_type: Some("long".to_string()),
                mean_type: None,
                ma_type: None,
                fast_period: fast,
                slow_period: slow,
                entry_threshold: entry,
                exit_threshold: exit,
                stop_loss: sl,
                take_profit: tp,
                position_size: size,
                risk_per_trade: None,
                max_concurrent_positions: None,
                slippage: None,
                commission: None,
                entry_delay: None,
                min_holding_period: None,
                max_holding_period: None,
                reversion_style: None,
                lookback_period: None,
                exit_z_score: None,
                entry_z_score: None,
                band_multiplier: None,
                cooldown_period: None,
            },
        )
        .collect()
}
