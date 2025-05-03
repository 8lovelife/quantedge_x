use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::engine::backtest_result::Balance;

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyReturnData {
    pub month: String,
    #[serde(rename = "strategyReturn")]
    pub strategy_return: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistributionData {
    pub bin: String,
    pub count: u64,
    #[serde(rename = "binValue")]
    pub bin_value: i64,
}

pub fn calculate_monthly_returns(balances: &[Balance]) -> Vec<MonthlyReturnData> {
    let mut monthly_groups: BTreeMap<(i32, u32), Vec<&Balance>> = BTreeMap::new();

    for balance in balances {
        let date = NaiveDate::parse_from_str(&balance.date[0..10], "%Y-%m-%d").unwrap();
        monthly_groups
            .entry((date.year(), date.month()))
            .or_default()
            .push(balance);
    }

    let mut results = Vec::new();

    for ((year, month), entries) in monthly_groups {
        if entries.len() < 2 {
            continue;
        }

        let start = entries.first().unwrap().capital;
        let end = entries.last().unwrap().capital;

        if start != 0.0 {
            let return_rate = (end - start) / start;
            results.push(MonthlyReturnData {
                month: format!("{:04}-{:02}", year, month),
                strategy_return: return_rate,
            });
        }
    }

    results
}

/// Calculate distribution of daily returns from balance history
pub fn calculate_daily_return_distribution(
    balances: &[Balance],
    bin_size_percent: f64, // e.g., 1.0 for 1% bins
) -> Vec<DistributionData> {
    let mut map: BTreeMap<String, &Balance> = BTreeMap::new();

    for b in balances {
        map.insert(b.date.clone(), b); // overwrites previous = keep last
    }

    let mut sorted_balances: Vec<Balance> = map.into_iter().map(|(_, b)| b.clone()).collect();

    // Sort by date if not already sorted
    sorted_balances.sort_by_key(|b| NaiveDate::parse_from_str(&b.date[0..10], "%Y-%m-%d").unwrap());

    let bin_size = bin_size_percent / 100.0;
    let mut bin_counts: BTreeMap<i64, u64> = BTreeMap::new();

    // Loop over daily pairs to compute returns
    for pair in sorted_balances.windows(2) {
        let prev = &pair[0];
        let curr = &pair[1];

        let prev_cap = prev.capital;
        let curr_cap = curr.capital;

        if prev_cap == 0.0 {
            continue; // avoid divide-by-zero
        }

        let daily_return = (curr_cap - prev_cap) / prev_cap;
        let bin_index = (daily_return / bin_size).floor() as i64;
        *bin_counts.entry(bin_index).or_insert(0) += 1;
    }

    // Format bin counts into DistributionData
    let mut result = Vec::new();
    for (bin_index, count) in bin_counts {
        let bin_start = bin_index as f64 * bin_size_percent;
        let bin_end = bin_start + bin_size_percent;
        let bin_label = format!("{:.0}% to {:.0}%", bin_start, bin_end);
        let bin_value = (bin_start + bin_end) as i64 / 2;

        result.push(DistributionData {
            bin: bin_label,
            count,
            bin_value,
        });
    }

    result
}
