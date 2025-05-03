use std::sync::{Arc, Mutex};

use duckdb::{Connection, params};
use serde_json::Value;

use crate::data::duckdb::schema::StrategyTemplate;

use super::connection::get_db_conn;

pub struct StrategyTemplateRepository {
    conn: Arc<Mutex<Connection>>,
}

impl StrategyTemplateRepository {
    pub fn new() -> Result<Self, duckdb::Error> {
        Ok(Self {
            conn: get_db_conn(),
        })
    }
    pub fn create_template(
        &self,
        input: StrategyTemplate,
    ) -> Result<StrategyTemplate, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"INSERT INTO strategy_template 
            (name, type, description, info, parameters, risk, execution, performance)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id"#,
        )?;

        let parameters_json = input
            .parameters
            .as_ref()
            .map(|v| serde_json::to_string(v))
            .transpose()
            .unwrap();

        let id: i64 = stmt.query_row(
            params![
                input.name,
                input.r#type,
                input.description,
                input.info,
                parameters_json,
                input.risk.as_ref().map(|v| v.to_string()),
                input.execution.as_ref().map(|v| v.to_string()),
                input.performance.as_ref().map(|v| v.to_string()),
            ],
            |row| row.get(0),
        )?;

        Ok(StrategyTemplate {
            id: Some(id),
            ..input
        })
    }

    pub fn get_template_by_id(&self, id: i64) -> Result<Option<StrategyTemplate>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, type, description, info, parameters, risk, execution, performance, likes, usage, author, latest_lab_backtest_version, created_at, updated_at 
             FROM strategy_template WHERE id = ?",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(StrategyTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                r#type: row.get(2)?,
                description: row.get(3)?,
                info: row.get(4)?,
                parameters: row
                    .get::<_, Option<String>>(5)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                risk: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                execution: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                performance: row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                likes: row.get(9)?,
                usage: row.get(10)?,
                author: row.get(11)?,
                latest_lab_backtest_version: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;

        match rows.next() {
            Some(template) => Ok(Some(template?)),
            None => Ok(None),
        }
    }

    pub fn delete_template_by_id(&self, id: i64) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let result = conn.execute("DELETE FROM strategy_template WHERE id = ?", params![id])?;

        Ok(result)
    }

    pub fn update_backtest_version_by_id(
        &self,
        id: i64,
        performance: &Value,
        latest_backtest_version: i64,
    ) -> Result<usize, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        println!(
            "conn {:?} id = {} latest_lab_backtest_version = {}, performance = {} ",
            conn, id, latest_backtest_version, performance
        );

        let result = conn.execute(
            r#"
            UPDATE strategy_template SET performance = ? , latest_lab_backtest_version = ? WHERE id = ?
            "#,
            params![performance.to_string(),latest_backtest_version, id],
        )?;

        Ok(result)
    }

    pub fn get_count(&self) -> Result<u32, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM strategy_template")?;
        let count: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_templates_by_page(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<StrategyTemplate>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();

        let offset = (page.saturating_sub(1) * page_size) as u64;
        let limit = page_size as u64;

        let mut stmt = conn.prepare(
            "SELECT id, name, type, description, info, parameters, risk, execution, performance, 
                    likes, usage, author, latest_lab_backtest_version, created_at, updated_at 
             FROM strategy_template
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?",
        )?;

        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(StrategyTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                r#type: row.get(2)?,
                description: row.get(3)?,
                info: row.get(4)?,
                parameters: row
                    .get::<_, Option<String>>(5)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                risk: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                execution: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                performance: row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                likes: row.get(9)?,
                usage: row.get(10)?,
                author: row.get(11)?,
                latest_lab_backtest_version: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(row?);
        }

        Ok(templates)
    }
}
