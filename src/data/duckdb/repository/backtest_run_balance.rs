use std::sync::{Arc, Mutex};

use duckdb::{Connection, params};

use crate::data::duckdb::schema::BacktestRunBalance;

use super::connection::get_db_conn;

pub struct BacktestRunBalanceRepository {
    conn: Arc<Mutex<Connection>>,
}

impl BacktestRunBalanceRepository {
    pub fn new() -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(
        &self,
        mut balance: BacktestRunBalance,
    ) -> Result<BacktestRunBalance, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
                INSERT INTO backtest_run_balance 
                (strategy_id, run_id, date, captial, trades)
                VALUES (?, ?, ?, ?, ?)
                RETURNING id
            "#,
        )?;
        let mut rows = stmt.query(params![
            balance.strategy_id,
            balance.run_id,
            balance.date,
            balance.capital,
            balance.trades,
        ])?;

        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            balance.id = Some(id);
            Ok(balance)
        } else {
            Err(duckdb::Error::QueryReturnedNoRows)
        }
    }

    pub fn get_by_id(&self, run_id: i64) -> Result<Option<BacktestRunBalance>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
                "SELECT id, strategy_id, run_id, date, capital, trades, created_at 
                FROM backtest_run_balance
                WHERE run_id = ?
                "#,
        )?;

        let run: Option<BacktestRunBalance> = stmt.query_row([run_id], |row| {
            Ok(Some(BacktestRunBalance {
                id: row.get(0)?,
                strategy_id: row.get(1)?,
                run_id: row.get(2)?,
                date: row.get(3)?,
                capital: row.get(4)?,
                trades: row.get(5)?,
                created_at: row.get(6)?,
            }))
        })?;

        Ok(run)
    }
}
