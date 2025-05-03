use duckdb::{Connection, Error, OptionalExt, params};
use std::sync::{Arc, Mutex};

use crate::data::duckdb::schema::LabRunHistory;

use super::connection::get_db_conn;

pub struct LabRunHistoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LabRunHistoryRepository {
    pub fn new() -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(&self, mut run: LabRunHistory) -> Result<LabRunHistory, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO lab_backtest_run_history
            (template_id, parameters, market_details, status, start_time)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )?;

        let end_time_str = run.end_time.as_ref().map(|dt| dt.0.to_rfc3339());
        let start_time_str = &run.start_time.0.clone().to_rfc3339();
        let mut rows = stmt.query(params![
            run.template_id,
            run.parameters.to_string(),
            run.market_details.to_string(),
            run.status,
            start_time_str,
        ])?;

        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            run.id = Some(id);
            Ok(run)
        } else {
            Err(duckdb::Error::QueryReturnedNoRows)
        }
    }

    pub fn get_latest_top_run(
        &self,
        template_id: i64,
        top: u32,
    ) -> Result<Vec<LabRunHistory>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, template_id, parameters, performance,status, start_time, end_time, created_at , market_details
            FROM lab_backtest_run_history
            WHERE template_id = ?
            ORDER BY start_time DESC
            LIMIT ?
            "#,
        )?;

        let runs = stmt
            .query_map(params![template_id, top], |row| {
                Ok(LabRunHistory {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    parameters: row
                        .get::<_, Option<String>>(2)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                    performance: row
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                    status: row.get(4)?,
                    start_time: row.get(5)?,
                    end_time: row.get(6)?,
                    created_at: row.get(7)?,
                    market_details: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<LabRunHistory>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, template_id, parameters, performance, status, start_time, end_time, created_at, market_details
            FROM lab_backtest_run_history
            WHERE id = ?
            "#,
        )?;

        let run: Option<LabRunHistory> = match stmt.query_row([id], |row| {
            Ok(LabRunHistory {
                id: row.get(0)?,
                template_id: row.get(1)?,
                parameters: row
                    .get::<_, Option<String>>(2)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                performance: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                status: row.get(4)?,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                created_at: row.get(7)?,
                market_details: row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
            })
        }) {
            Ok(v) => Ok(Some(v)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }?;

        Ok(run)
    }

    pub fn update(&self, run: &LabRunHistory) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            UPDATE lab_backtest_run_history
            SET template_id = ?, parameters = ?,performance = ?, status = ?, start_time = ?, end_time = ?, market_details = ?
            WHERE id = ?
            "#,
            params![
                run.template_id,
                run.parameters.to_string(),
                run.performance.as_ref().map(|p| p.to_string()),
                run.status,
                run.start_time.0.to_rfc3339(),
                run.end_time.as_ref().map(|dt| dt.0.to_rfc3339()),
                run.market_details.to_string(),
                run.id
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            DELETE FROM lab_backtest_run_history
            WHERE id = ?
            "#,
            [id],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<LabRunHistory>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, template_id, parameters, performance, status, start_time, end_time, created_at, market_details
            FROM lab_backtest_run_history
            ORDER BY start_time DESC
            "#,
        )?;

        let runs = stmt
            .query_map([], |row| {
                Ok(LabRunHistory {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    parameters: row
                        .get::<_, Option<String>>(2)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                    performance: row
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                    status: row.get(4)?,
                    start_time: row.get(5)?,
                    end_time: row.get(6)?,
                    created_at: row.get(7)?,
                    market_details: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    pub fn get_by_page(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<LabRunHistory>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, template_id, parameters,performance, status, start_time, end_time, created_at, market_details
            FROM lab_backtest_run_history
            ORDER BY start_time DESC
            LIMIT ? OFFSET ?
            "#,
        )?;

        let runs = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                Ok(LabRunHistory {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    parameters: row
                        .get::<_, Option<String>>(2)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                    performance: row
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                    status: row.get(4)?,
                    start_time: row.get(5)?,
                    end_time: row.get(6)?,
                    created_at: row.get(7)?,
                    market_details: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::duckdb::schema::backtest_run_history::BacktestRun;

    use super::*;
    use chrono::{TimeZone, Utc};

    // #[test]
    // fn test_create_backtest_run_instance() {
    //     let run = BacktestRun {
    //         id: Some(1),
    //         strategy_id: 42,
    //         parameters: r#"{"leverage":5,"symbol":"BTC"}"#.to_string(),
    //         status: "running".to_string(),
    //         start_time: crate::data::duckdb::types::Timestamp(
    //             Utc.with_ymd_and_hms(2025, 3, 21, 12, 0, 0).unwrap(),
    //         ),
    //         end_time: None,
    //         created_at: None,
    //     };
    //     assert_eq!(run.id, Some(1));
    //     assert_eq!(run.strategy_id, 42);
    //     assert_eq!(run.parameters, r#"{"leverage":5,"symbol":"BTC"}"#);
    //     assert_eq!(run.status, "running");
    //     assert_eq!(
    //         run.start_time.0,
    //         Utc.with_ymd_and_hms(2025, 3, 21, 12, 0, 0).unwrap(),
    //     );
    //     assert!(run.end_time.is_none());
    // }
}
