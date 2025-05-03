use crate::data::duckdb::schema::Ohlcv;
use chrono::{DateTime, Utc};
use duckdb::{Connection, params};
use std::sync::{Arc, Mutex};

use super::connection::get_db_conn;

pub struct OhlcvRepository {
    conn: Arc<Mutex<Connection>>,
}

impl OhlcvRepository {
    pub fn new(db_path: Option<String>) -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(&self, ohlcv: &Ohlcv) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO ohlcv_data (symbol, timestamp, open, high, low, close, volume)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                &ohlcv.symbol,
                &ohlcv.timestamp.0.to_rfc3339(),
                &ohlcv.open,
                &ohlcv.high,
                &ohlcv.low,
                &ohlcv.close,
                &ohlcv.volume,
            ],
        )?;
        Ok(())
    }

    pub fn batch_insert(&self, ohlcvs: &[Ohlcv]) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT INTO ohlcv_data (symbol, timestamp, open, high, low, close, volume)
            VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;

        for ohlcv in ohlcvs {
            stmt.execute(params![
                &ohlcv.symbol,
                &ohlcv.timestamp.0.to_rfc3339(),
                &ohlcv.open,
                &ohlcv.high,
                &ohlcv.low,
                &ohlcv.close,
                &ohlcv.volume,
            ])?;
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Ohlcv>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT symbol, timestamp, open, high, low, close, volume
            FROM ohlcv_data
            WHERE id = ?
            "#,
        )?;

        let ohlcv: Option<Ohlcv> = stmt.query_row([id], |row| {
            Ok(Some(Ohlcv {
                symbol: row.get(0)?,
                timestamp: row.get(1)?,
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                volume: row.get(6)?,
            }))
        })?;

        Ok(ohlcv)
    }

    pub fn get_by_symbol_and_timestamp(
        &self,
        symbol: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<Ohlcv>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT symbol, timestamp, open, high, low, close, volume
            FROM ohlcv_data
            WHERE symbol = ? AND timestamp = ?
            "#,
        )?;

        let ohlcv: Option<Ohlcv> = stmt.query_row([symbol, &timestamp.to_rfc3339()], |row| {
            Ok(Some(Ohlcv {
                symbol: row.get(0)?,
                timestamp: row.get(1)?,
                open: row.get(2)?,
                high: row.get(3)?,
                low: row.get(4)?,
                close: row.get(5)?,
                volume: row.get(6)?,
            }))
        })?;

        Ok(ohlcv)
    }

    pub fn get_by_symbol(&self, symbol: &str) -> Result<Vec<Ohlcv>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT symbol, timestamp, open, high, low, close, volume
            FROM ohlcv_data
            WHERE symbol = ?
            ORDER BY timestamp
            "#,
        )?;

        let ohlcv_data = stmt
            .query_map([symbol], |row| {
                Ok(Ohlcv {
                    symbol: row.get(0)?,
                    timestamp: row.get(1)?,
                    open: row.get(2)?,
                    high: row.get(3)?,
                    low: row.get(4)?,
                    close: row.get(5)?,
                    volume: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ohlcv_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::duckdb::types::Timestamp;

    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> (tempfile::TempDir, OhlcvRepository) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir
            .path()
            .join("test.db")
            .to_str()
            .unwrap()
            .to_string();
        let repo = OhlcvRepository::new(Some(db_path.clone())).unwrap();

        // Create trades table
        repo.conn
            .lock()
            .unwrap()
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS trades (
                id TEXT PRIMARY KEY,
                strategy TEXT NOT NULL,
                type TEXT NOT NULL,
                asset TEXT NOT NULL,
                amount TEXT NOT NULL,
                price TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                status TEXT NOT NULL,
                profit TEXT
            )
            "#,
                [],
            )
            .unwrap();

        // Create ohlcv table
        repo.conn
            .lock()
            .unwrap()
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS ohlcv (
                id TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                open TEXT NOT NULL,
                high TEXT NOT NULL,
                low TEXT NOT NULL,
                close TEXT NOT NULL,
                volume TEXT NOT NULL,
                UNIQUE(symbol, timestamp)
            )
            "#,
                [],
            )
            .unwrap();

        (temp_dir, repo)
    }

    fn create_test_ohlcv() -> Ohlcv {
        Ohlcv {
            symbol: "BTC".to_string(),
            timestamp: Timestamp(Utc::now()),
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 100.0,
        }
    }
}
