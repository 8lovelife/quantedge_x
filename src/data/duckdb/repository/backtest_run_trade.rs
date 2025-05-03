use std::sync::{Arc, Mutex};

use duckdb::{Connection, params};

use crate::{data::duckdb::schema::BacktestRunTrade, utils::get_db_path_str};

use super::connection::get_db_conn;

pub struct BacktestRunTradeRepository {
    conn: Arc<Mutex<Connection>>,
}

impl BacktestRunTradeRepository {
    pub fn new() -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(&self, mut trade: BacktestRunTrade) -> Result<BacktestRunTrade, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
                INSERT INTO backtest_run_trade 
                (strategy_id, run_id, date, trade_type, result, profit)
                VALUES (?, ?, ?, ?, ?, ?)
                RETURNING id
            "#,
        )?;
        let mut rows = stmt.query(params![
            trade.strategy_id,
            trade.run_id,
            trade.date,
            trade.trade_type,
            trade.result,
            trade.profit,
        ])?;

        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            trade.id = Some(id);
            Ok(trade)
        } else {
            Err(duckdb::Error::QueryReturnedNoRows)
        }
    }

    pub fn get_by_id(&self, run_id: i64) -> Result<Option<BacktestRunTrade>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
                "SELECT id, strategy_id, run_id, date, trade_type, result, profit, created_at 
                FROM backtest_run_trade
                WHERE run_id = ?
                "#,
        )?;

        let run: Option<BacktestRunTrade> = stmt.query_row([run_id], |row| {
            Ok(Some(BacktestRunTrade {
                id: row.get(0)?,
                strategy_id: row.get(1)?,
                run_id: row.get(2)?,
                date: row.get(3)?,
                trade_type: row.get(4)?,
                result: row.get(5)?,
                profit: row.get(6)?,
                created_at: row.get(7)?,
            }))
        })?;

        Ok(run)
    }
}
