use std::sync::{Arc, Mutex};

use duckdb::{Connection, params};

use crate::{data::duckdb::schema::BacktestRunMetrics, utils::get_db_path_str};

use super::connection::get_db_conn;

pub struct BacktestRunMetricsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl BacktestRunMetricsRepository {
    pub fn new() -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(
        &self,
        mut metrics: BacktestRunMetrics,
    ) -> Result<BacktestRunMetrics, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
                INSERT INTO backtest_run_metrics 
                (strategy_id, run_id, date,strategy_return,max_drawdown,win_rate,sharpe_ratio)
                VALUES (?, ?, ?, ?,?,?,?)
                RETURNING id
            "#,
        )?;
        let mut rows = stmt.query(params![
            metrics.strategy_id,
            metrics.run_id,
            metrics.date,
            metrics.strategy_return,
            metrics.max_drawdown,
            metrics.win_rate,
            metrics.sharpe_ratio,
        ])?;

        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            metrics.id = Some(id);
            Ok(metrics)
        } else {
            Err(duckdb::Error::QueryReturnedNoRows)
        }
    }

    pub fn get_by_id(&self, run_id: i64) -> Result<Option<BacktestRunMetrics>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
                "SELECT id, strategy_id, run_id, date, strategy_return,max_drawdown,win_rate,sharpe_ratio, created_at 
                FROM backtest_run_metrics
                WHERE run_id = ?
                "#,
        )?;

        let run: Option<BacktestRunMetrics> = stmt.query_row([run_id], |row| {
            Ok(Some(BacktestRunMetrics {
                id: row.get(0)?,
                strategy_id: row.get(1)?,
                run_id: row.get(2)?,
                date: row.get(3)?,
                strategy_return: row.get(4)?,
                max_drawdown: row.get(5)?,
                win_rate: row.get(6)?,
                sharpe_ratio: row.get(7)?,
                created_at: row.get(8)?,
            }))
        })?;

        Ok(run)
    }
}
