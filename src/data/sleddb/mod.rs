pub mod chart_db;
pub mod user_db_manager;

pub use chart_db::ChartDB;

#[cfg(test)]
mod tests {
    use crate::service::backtest_service::RunBacktestData;

    use super::ChartDB;

    #[test]
    fn test_chart_db() {
        let chart_db = ChartDB::new().unwrap();
        let chart_json: Option<RunBacktestData> = chart_db.retrieve(&"1-8-fortest").unwrap();

        if let Some(chart) = chart_json {
            print!("chart json {:?}", chart);
        }
    }
}
