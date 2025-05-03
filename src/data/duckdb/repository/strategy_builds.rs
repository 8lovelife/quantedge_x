use std::sync::{Arc, Mutex};

use duckdb::{Connection, params};
use serde_json::Value;

use crate::data::duckdb::schema::{StrategyBuilds, strategy_builds::ProgressInType};

use super::connection::{get_db_conn, get_user_connection_manager};

pub struct StrategyBuildsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl StrategyBuildsRepository {
    pub fn build(user_id: i64) -> Self {
        Self {
            conn: get_user_connection_manager().get_connection(user_id),
        }
    }

    // pub fn new(db_path: Option<String>) -> Result<Self, duckdb::Error> {
    //     Ok(Self {
    //         conn: get_db_conn(),
    //     })
    // }

    pub fn create(&self, algorithm_type: ProgressInType) -> Result<ProgressInType, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "INSERT INTO strategy_builds 
                (name, description, algorithm_type, progress) 
             VALUES (?, ?, ?, 'type') 
             RETURNING id",
        )?;

        let id: i64 = stmt.query_row(
            params![
                algorithm_type.name,
                algorithm_type.description,
                algorithm_type.algorithm_type,
            ],
            |row| row.get(0),
        )?;

        Ok(ProgressInType {
            id: Some(id),
            ..algorithm_type
        })
    }

    pub fn get_status(&self, id: i64) -> Result<(String, String), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT progress, lifecycle FROM strategy_builds WHERE id = ?")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let progress: String = row.get(0)?;
            let lifecycle: String = row.get(1)?;
            Ok((progress, lifecycle))
        } else {
            Err(duckdb::Error::FromSqlConversionFailure(
                0,
                duckdb::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Strategy not found",
                )),
            ))
        }
    }

    pub fn update_type_stage(
        &self,
        id: i64,
        input: &ProgressInType,
        next_stage: &'static str,
    ) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE strategy_builds 
             SET name = ?, 
                 description = ?, 
                 algorithm_type = ?, 
                 progress = ?,
             WHERE id = ?",
            params![
                input.name,
                input.description,
                input.algorithm_type,
                next_stage,
                id
            ],
        )?;

        Ok(())
    }

    pub fn update_parameters(
        &self,
        id: i64,
        parameters: &Value,
        next_stage: &'static str,
    ) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE strategy_builds SET parameters = ?, progress = ? WHERE id = ?",
            params![parameters.to_string(), next_stage, id],
        )?;

        Ok(())
    }

    pub fn update_assets(
        &self,
        id: i64,
        assets: &Value,
        next_stage: &'static str,
    ) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE strategy_builds SET assets = ?, progress = ? WHERE id = ?",
            params![assets.to_string(), next_stage, id],
        )?;

        Ok(())
    }

    pub fn update_risk(
        &self,
        id: i64,
        risk: &Value,
        next_stage: &'static str,
    ) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE strategy_builds SET risk = ?,progress = ? WHERE id = ?",
            params![risk.to_string(), next_stage, id],
        )?;

        Ok(())
    }

    pub fn get_by_page(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<StrategyBuilds>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let offset = (page.saturating_sub(1) * page_size) as u64;
        let limit = page_size as u64;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, algorithm_type, algorithm_sub_type, lifecycle, progress,
                   created_at, updated_at, parameters, risk, assets,latest_backtest_version,
                   backtest_performance,paper_performance,live_performance,
                   apply_backtest_version, market_details
            FROM strategy_builds
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
        "#,
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(StrategyBuilds {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm_type: row.get(3)?,
                algorithm_sub_type: row.get(4)?,
                lifecycle: row.get(5)?,
                progress: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                parameters: row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                risk: row
                    .get::<_, Option<String>>(10)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                assets: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                latest_backtest_version: row.get(12)?,
                backtest_performance: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                paper_performance: row
                    .get::<_, Option<String>>(14)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                live_performance: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                apply_backtest_version: row.get(16)?,
                market_details: row
                    .get::<_, Option<String>>(17)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;

        let mut strategies = Vec::new();
        for strategy in rows {
            strategies.push(strategy?);
        }

        Ok(strategies)
    }

    pub fn get_by_status_page(
        &self,
        page: u32,
        page_size: u32,
        status: &str,
    ) -> Result<Vec<StrategyBuilds>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let offset = (page.saturating_sub(1) * page_size) as u64;
        let limit = page_size as u64;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, algorithm_type, algorithm_sub_type, lifecycle, progress,
                   created_at, updated_at, parameters, risk, assets, latest_backtest_version, 
                   backtest_performance, paper_performance, live_performance,
                   apply_backtest_version, market_details
            FROM strategy_builds where lifecycle = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
        "#,
        )?;

        let rows = stmt.query_map(params![status, limit, offset], |row| {
            Ok(StrategyBuilds {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm_type: row.get(3)?,
                algorithm_sub_type: row.get(4)?,
                lifecycle: row.get(5)?,
                progress: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                parameters: row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                risk: row
                    .get::<_, Option<String>>(10)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                assets: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                latest_backtest_version: row.get(12)?,
                backtest_performance: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                paper_performance: row
                    .get::<_, Option<String>>(14)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                live_performance: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                apply_backtest_version: row.get(16)?,
                market_details: row
                    .get::<_, Option<String>>(17)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;

        let mut strategies = Vec::new();
        for strategy in rows {
            strategies.push(strategy?);
        }

        Ok(strategies)
    }

    pub fn delete_by_id(&self, id: i64) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            r#"
            DELETE FROM strategy_builds WHERE id = ?
            "#,
            [id],
        )?;

        Ok(result)
    }

    pub fn get_count(&self) -> Result<u32, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM strategy_builds")?;
        let count: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_count_by_status(&self, status: &str) -> Result<u32, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM strategy_builds where lifecycle = ?")?;
        let count: u32 = stmt.query_row(params![status], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<StrategyBuilds>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, algorithm_type, algorithm_sub_type, lifecycle, progress,
                   created_at, updated_at, parameters, risk, assets, latest_backtest_version, 
                   backtest_performance, paper_performance, live_performance,
                   apply_backtest_version,market_details
            FROM strategy_builds 
            WHERE id = ?
            "#,
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(StrategyBuilds {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm_type: row.get(3)?,
                algorithm_sub_type: row.get(4)?,
                lifecycle: row.get(5)?,
                progress: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                parameters: row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                risk: row
                    .get::<_, Option<String>>(10)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                assets: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                latest_backtest_version: row.get(12)?,
                backtest_performance: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                paper_performance: row
                    .get::<_, Option<String>>(14)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                live_performance: row
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                apply_backtest_version: row.get(16)?,
                market_details: row
                    .get::<_, Option<String>>(17)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
            })
        })?;

        match rows.next() {
            Some(strategy) => Ok(Some(strategy?)),
            None => Ok(None),
        }
    }

    pub fn update_lasest_backtest_by_id(
        &self,
        id: i64,
        latest_backtest_version: i64,
    ) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        println!(
            "conn {:?} id = {} latest_backtest_version = {}",
            conn, id, latest_backtest_version
        );

        let result = conn.execute(
            r#"
            UPDATE strategy_builds SET latest_backtest_version = ? WHERE id = ?
            "#,
            params![latest_backtest_version, id],
        )?;

        Ok(result)
    }

    pub fn update_apply_backtest_by_id(
        &self,
        id: i64,
        apply_backtest_version: i64,
        parameters: &Value,
        risk: &Value,
        backtest_performance: &Value,
        market_details: &Value,
    ) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        println!(
            "conn {:?} id = {} apply_backtest_version = {}",
            conn, id, apply_backtest_version
        );

        let result = conn.execute(
            r#"
            UPDATE strategy_builds SET parameters = ?, risk = ?, backtest_performance = ?, market_details = ?, apply_backtest_version = ?, lifecycle = 'backtest'  WHERE id = ?
            "#,
            params![parameters.to_string(),risk.to_string(),backtest_performance.to_string(),market_details.to_string(),apply_backtest_version, id],
        )?;

        Ok(result)
    }
}
