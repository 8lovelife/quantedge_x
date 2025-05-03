use crate::data::duckdb::schema::Trade;
use crate::utils::get_db_path_str;
use chrono::{DateTime, Utc};
use duckdb::Connection;
use std::sync::{Arc, Mutex};

use super::connection::get_db_conn;

pub struct TradeRepository {
    conn: Arc<Mutex<Connection>>,
}

impl TradeRepository {
    pub fn new(db_path: Option<String>) -> Result<Self, duckdb::Error> {
        // let db_path = db_path.unwrap_or_else(get_db_path_str);
        // let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: get_db_conn(),
        })
    }

    pub fn create(&self, trade: &Trade) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO trades (id, strategy, type, asset, amount, price, timestamp, status, profit)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            [
                &trade.id,
                &trade.strategy,
                &trade.trade_type,
                &trade.asset,
                &trade.amount.to_string(),
                &trade.price.to_string(),
                &trade.timestamp.to_rfc3339(),
                &trade.status,
                &trade.profit.map(|p| p.to_string()).unwrap_or_default(),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Trade>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, strategy, type, asset, amount, price, timestamp, status, profit
            FROM trades
            WHERE id = ?
            "#,
        )?;

        let trade: Option<Trade> = stmt.query_row([id], |row| {
            // Get the profit value safely
            let profit_str: Option<String> = row.get(8)?;
            let profit = profit_str.and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    s.parse::<f64>().ok()
                }
            });

            Ok(Some(Trade {
                id: row.get(0)?,
                strategy: row.get(1)?,
                trade_type: row.get(2)?,
                asset: row.get(3)?,
                amount: row.get::<_, String>(4)?.parse().unwrap(),
                price: row.get::<_, String>(5)?.parse().unwrap(),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
                status: row.get(7)?,
                profit,
            }))
        })?;

        Ok(trade)
    }

    pub fn get_all(&self) -> Result<Vec<Trade>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, strategy, type, asset, amount, price, timestamp, status, profit
            FROM trades
            "#,
        )?;

        let trades = stmt
            .query_map([], |row| {
                // Get the profit value safely
                let profit_str: Option<String> = row.get(8)?;
                let profit = profit_str.and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        s.parse::<f64>().ok()
                    }
                });

                Ok(Trade {
                    id: row.get(0)?,
                    strategy: row.get(1)?,
                    trade_type: row.get(2)?,
                    asset: row.get(3)?,
                    amount: row.get::<_, String>(4)?.parse().unwrap(),
                    price: row.get::<_, String>(5)?.parse().unwrap(),
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    status: row.get(7)?,
                    profit,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(trades)
    }

    pub fn update(&self, trade: &Trade) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            UPDATE trades
            SET strategy = ?,
                type = ?,
                asset = ?,
                amount = ?,
                price = ?,
                timestamp = ?,
                status = ?,
                profit = ?
            WHERE id = ?
            "#,
            [
                &trade.strategy,
                &trade.trade_type,
                &trade.asset,
                &trade.amount.to_string(),
                &trade.price.to_string(),
                &trade.timestamp.to_rfc3339(),
                &trade.status,
                &trade.profit.map(|p| p.to_string()).unwrap_or_default(),
                &trade.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            DELETE FROM trades
            WHERE id = ?
            "#,
            [id],
        )?;
        Ok(())
    }

    pub fn get_recent_trades(&self, page: u32, limit: u32) -> Result<Vec<Trade>, duckdb::Error> {
        let conn = self.conn.lock().unwrap();
        let offset = (page - 1) * limit;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, strategy, type, asset, amount, price, timestamp, status, profit
            FROM trades
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )?;

        let trades = stmt
            .query_map([limit.to_string(), offset.to_string()], |row| {
                // Get the profit value safely
                let profit_str: Option<String> = row.get(8)?;
                let profit = profit_str.and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        s.parse::<f64>().ok()
                    }
                });

                Ok(Trade {
                    id: row.get(0)?,
                    strategy: row.get(1)?,
                    trade_type: row.get(2)?,
                    asset: row.get(3)?,
                    amount: row.get::<_, String>(4)?.parse().unwrap(),
                    price: row.get::<_, String>(5)?.parse().unwrap(),
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    status: row.get(7)?,
                    profit,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(trades)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> (tempfile::TempDir, TradeRepository) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir
            .path()
            .join("test.db")
            .to_str()
            .unwrap()
            .to_string();
        let repo = TradeRepository::new(Some(db_path)).unwrap();
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
        (temp_dir, repo)
    }

    fn create_test_trade() -> Trade {
        Trade {
            id: "test-trade-1".to_string(),
            strategy: "Test Strategy".to_string(),
            trade_type: "buy".to_string(),
            asset: "BTC".to_string(),
            amount: 0.1,
            price: 50000.0,
            timestamp: DateTime::parse_from_rfc3339("2024-03-13T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            status: "completed".to_string(),
            profit: Some(100.0),
        }
    }

    #[test]
    fn test_create_and_get_trade() {
        let (_temp_dir, repo) = setup_test_db();
        let trade = create_test_trade();

        // Test create
        repo.create(&trade).unwrap();

        // Test get_by_id
        let retrieved = repo.get_by_id(&trade.id).unwrap().unwrap();
        assert_eq!(retrieved.id, trade.id);
        assert_eq!(retrieved.strategy, trade.strategy);
        assert_eq!(retrieved.trade_type, trade.trade_type);
        assert_eq!(retrieved.asset, trade.asset);
        assert_eq!(retrieved.amount, trade.amount);
        assert_eq!(retrieved.price, trade.price);
        assert_eq!(retrieved.timestamp, trade.timestamp);
        assert_eq!(retrieved.status, trade.status);
        assert_eq!(retrieved.profit, trade.profit);
    }

    #[test]
    fn test_get_all_trades() {
        let (_temp_dir, repo) = setup_test_db();
        let trade1 = create_test_trade();
        let mut trade2 = create_test_trade();
        trade2.id = "test-trade-2".to_string();
        trade2.amount = 0.2;

        repo.create(&trade1).unwrap();
        repo.create(&trade2).unwrap();

        let all_trades = repo.get_all().unwrap();
        assert_eq!(all_trades.len(), 2);
        assert!(all_trades.iter().any(|t| t.id == trade1.id));
        assert!(all_trades.iter().any(|t| t.id == trade2.id));
    }

    #[test]
    fn test_update_trade() {
        let (_temp_dir, repo) = setup_test_db();
        let mut trade = create_test_trade();

        repo.create(&trade).unwrap();

        // Update trade
        trade.amount = 0.15;
        trade.profit = Some(150.0);
        repo.update(&trade).unwrap();

        // Verify update
        let updated = repo.get_by_id(&trade.id).unwrap().unwrap();
        assert_eq!(updated.amount, 0.15);
        assert_eq!(updated.profit, Some(150.0));
    }

    #[test]
    fn test_delete_trade() {
        let (_temp_dir, repo) = setup_test_db();
        let trade = create_test_trade();

        repo.create(&trade).unwrap();
        repo.delete(&trade.id).unwrap();

        // Verify deletion
        let deleted = repo.get_by_id(&trade.id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_get_nonexistent_trade() {
        let (_temp_dir, repo) = setup_test_db();
        let result = repo.get_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }
}
