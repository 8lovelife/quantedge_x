use std::sync::{Arc, Mutex};

use duckdb::Connection;

use crate::data::duckdb::schema::TradeStrategy;

use super::connection::get_db_conn;

pub struct TradeStrategyRepository {
    conn: Arc<Mutex<Connection>>,
}

impl TradeStrategyRepository {
    pub fn build(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    pub fn new(db_path: Option<String>) -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(
        &self,
        mut trade_strategy: TradeStrategy,
    ) -> Result<TradeStrategy, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO trade_strategy (name, description, algorithm, risk, allocation, timeframe, assets, status, parameters)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id
            "#,
        )?;

        let mut rows = stmt.query([
            &trade_strategy.name,
            &trade_strategy.description,
            &trade_strategy.algorithm,
            &trade_strategy.risk,
            &trade_strategy.allocation.to_string(),
            &trade_strategy.timeframe,
            &trade_strategy.assets,
            &trade_strategy.status,
            &trade_strategy.parameters,
        ])?;

        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            trade_strategy.id = Some(id);
            Ok(trade_strategy)
        } else {
            Err(duckdb::Error::QueryReturnedNoRows)
        }
    }

    pub fn get_all(&self) -> Result<Vec<TradeStrategy>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, algorithm, risk, allocation, timeframe, assets, status, parameters,latest_backtest_version
            FROM trade_strategy
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TradeStrategy {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm: row.get(3)?,
                risk: row.get(4)?,
                allocation: row.get(5)?,
                timeframe: row.get(6)?,
                assets: row.get(7)?,
                status: row.get(8)?,
                parameters: row.get(9)?,
                latest_backtest_version: row.get(10)?,
            })
        })?;

        let mut strategies = Vec::new();
        for strategy in rows {
            strategies.push(strategy?);
        }

        Ok(strategies)
    }

    pub fn get_by_page(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<TradeStrategy>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let offset = (page - 1) * page_size;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, algorithm, risk, allocation, timeframe, assets, status, parameters,latest_backtest_version
            FROM trade_strategy
            ORDER BY id
            LIMIT ? OFFSET ?
            "#,
        )?;

        let rows = stmt.query_map([page_size as i64, offset as i64], |row| {
            Ok(TradeStrategy {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm: row.get(3)?,
                risk: row.get(4)?,
                allocation: row.get(5)?,
                timeframe: row.get(6)?,
                assets: row.get(7)?,
                status: row.get(8)?,
                parameters: row.get(9)?,
                latest_backtest_version: row.get(10)?,
            })
        })?;

        let mut strategies = Vec::new();
        for strategy in rows {
            strategies.push(strategy?);
        }

        Ok(strategies)
    }

    pub fn get_count(&self) -> Result<u32, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM trade_strategy")?;
        let count: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<TradeStrategy>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, algorithm, risk, allocation, timeframe, assets, status, parameters,latest_backtest_version
            FROM trade_strategy 
            WHERE id = ?
            "#,
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(TradeStrategy {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm: row.get(3)?,
                risk: row.get(4)?,
                allocation: row.get(5)?,
                timeframe: row.get(6)?,
                assets: row.get(7)?,
                status: row.get(8)?,
                parameters: row.get(9)?,
                latest_backtest_version: row.get(10)?,
            })
        })?;

        match rows.next() {
            Some(strategy) => Ok(Some(strategy?)),
            None => Ok(None),
        }
    }

    pub fn update_by_id(
        &self,
        id: i64,
        trade_strategy: TradeStrategy,
    ) -> Result<TradeStrategy, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            r#"
            UPDATE trade_strategy 
            SET name = ?, description = ?, algorithm = ?, risk = ?, allocation = ?, 
                timeframe = ?, assets = ?, status = ?, parameters = ?
            WHERE id = ?
            "#,
            [
                &trade_strategy.name,
                &trade_strategy.description,
                &trade_strategy.algorithm,
                &trade_strategy.risk,
                &trade_strategy.allocation.to_string(),
                &trade_strategy.timeframe,
                &trade_strategy.assets,
                &trade_strategy.status,
                &trade_strategy.parameters,
                &id.to_string(),
            ],
        )?;

        Ok(trade_strategy)
    }

    pub fn delete_by_id(&self, id: i64) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            r#"
            DELETE FROM trade_strategy WHERE id = ?
            "#,
            [id],
        )?;

        Ok(result)
    }

    pub fn update_status_by_id(&self, id: i64, status: &str) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            r#"
            UPDATE trade_strategy SET status = ? WHERE id = ?
            "#,
            [status, &id.to_string()],
        )?;

        Ok(result)
    }

    pub fn update_backtest_version_by_id(
        &self,
        id: i64,
        latest_backtest_version: i64,
    ) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        println!(
            "conn {:?} id = {} latest_backtest_version = {}, ",
            conn, id, latest_backtest_version
        );

        let result = conn.execute(
            r#"
            UPDATE trade_strategy SET latest_backtest_version = ? WHERE id = ?
            "#,
            [latest_backtest_version, id],
        )?;

        Ok(result)
    }
}
